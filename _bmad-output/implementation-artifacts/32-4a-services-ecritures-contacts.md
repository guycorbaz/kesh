# Story 32.4a : la couche de services — rollout des lectures : écritures comptables, postes ouverts, contacts

Status: review

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-4a`, branche `story/32-4a` partie de `main`
`89d68704` (la 32-3 fusionnée). Registre : `epic-32-choix-autonomes-324a.md` (C-32-4a-1 à C-32-4a-12). Clé courte :
`324a`. Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs` ; issue des LOW de la 32-3 : #738, en
`refs` (A5, P2-2, et B4 pour sa seule forme coupée, C-32-4a-10) ; défaut de plage des dates laissé hors périmètre :
#742, ouverte. Les numéros de ligne cités sont ceux de `89d68704`.*

## Story

En tant que Guy, qui branchera un agent sur ses livres réels par le serveur MCP de la 0.16.0, je veux que les écritures
comptables, les postes ouverts d'un compte et le carnet d'adresses se lisent **par les mêmes services** pour l'API et
pour le MCP — contrôle d'accès, validation de la demande et **bornes de pagination** compris —, afin que l'outil MCP de
la 32-9a et de la 32-9b ne puisse ni contourner une règle écrite dans un handler, ni lire une page sans plafond.

## Origine

- `planning-artifacts/epic-32-serveur-mcp.md` § *32-4* : rollout **mécanique** du motif de la 32-3 aux seules
  **lectures** que serviront les outils (C-32-0-4) ; les écritures restent dans les handlers jusqu'à la 0.17.0 ; revue
  fichier par fichier. **AC de l'epic** : tests d'API inchangés et verts ; chaque service porte son contrôle d'accès ;
  inventaire clos des handlers de lecture des modules visés ; **risque nommé** : une liste paginée dont les bornes se
  perdent dans le déplacement.
- **Le motif** : fiche `32-3-services-motif.md` (§ *Le motif*, règles R1 à R8) et le doc-comment de
  `crates/kesh-api/src/services/mod.rs` (l. 1-98), qui les porte pour le rollout. Une lecture **sans trace** rend
  directement (R5, comme `plan_comptable::lister`) : aucune des cinq lectures de cette story n'écrit au journal d'audit
  (fait 6), donc aucun `Rapport`, aucune `Trace`.
- Les outils consommateurs : 32-9a (« écritures (liste, détail) […] postes ouverts », portée `books:read`) et 32-9b
  (« contacts (liste, détail) », portée `contacts:read`).

## Ce que le code dit (relevé au code le 2026-10-24, `89d68704`)

1. **Les trois fichiers visés portent 17 handlers, dont 7 lectures** (`grep -n '^pub async fn'`) :
   `routes/journal_entries.rs` 7 (`list_journal_entries` l. 345, `get_journal_entry` l. 489 ; cinq mutants l. 540,
   571, 689, 772, 830), `routes/letterings.rs` 5 (`get_lettering` l. 236, `get_open_items` l. 534,
   `get_lettering_proposals` l. 557 ; deux mutants l. 195, 258), `routes/contacts.rs` 5 (`list_contacts` l. 563,
   `get_contact` l. 607 ; trois mutants l. 628, 691, 758). **Les postes ouverts** sont le handler
   `routes::letterings::get_open_items`, monté sur `GET /api/v1/accounts/{id}/open-items` (`lib.rs:866-869`). Les
   **17** corps de handler portent tous `state.pool` (relevé par `awk` sur les corps, avant la ligne `#[cfg(test)]`).
2. **RBAC** : les sept lectures sont dans `authenticated_routes` (tous rôles, `lib.rs:848-880`) ; leurs mutations dans
   `comptable_routes` (l. 406 et suivantes). L'exigence API des cinq services est donc `Consultation`.
3. **Pagination** — trois listes, trois bornes, et **deux dépôts sur trois n'écrêtent rien** :
   - écritures : `DEFAULT_LIMIT` 50, `MAX_LIMIT` 500 (`journal_entries.rs:41-42`), défauts serde du DTO
     (`default_limit` l. 44, `default_offset` l. 48), écrêtage `clamp(1, MAX_LIMIT)` et `offset.max(0)` (l. 352-353) ;
     le dépôt écrête **aussi** (`kesh-db/src/repositories/journal_entries.rs:935-936`, son `MAX_LIMIT` = 500 l. 777) ;
   - postes ouverts : `OPEN_ITEMS_DEFAULT_LIMIT` 50, `MAX_LIMIT` 500, **plafond commun** avec les propositions
     (`letterings.rs:286-292`), défaut serde `limit` (l. 309) et `offset` `#[serde(default)]` (l. 311), écrêtage l.
     541-542 ; le dépôt **n'écrête pas** — son doc-comment le dit : « l'appelant les écrête »
     (`kesh-db/src/repositories/letterings/open_items.rs:337-339`) ;
   - contacts : `MAX_LIST_LIMIT` 100, `DEFAULT_LIST_LIMIT` 20 (`contacts.rs:42-43`), DTO à `Option<i64>` (l. 66-69),
     défaut et écrêtage **dans le corps du handler** (l. 572-576) ; le dépôt lie `query.limit` et `query.offset` tels
     quels (`kesh-db/src/repositories/contacts.rs:472-474`).
   ⚠️ L'epic illustre le risque par « `clamp(1, 200)` » : aucune des trois listes ne porte ce plafond (500, 500, 100).
   Une borne perdue au déplacement des contacts ou des postes ouverts laisserait donc passer un `LIMIT` sans plafond, ou
   un `LIMIT`/`OFFSET` négatif (refus SQL, 500). La couverture actuelle : les bornes des postes ouverts sont figées par
   la route (`tests/open_items_e2e.rs:274-323` ; `:529-539` fige celles des **propositions**, le plafond commun que
   l'AC5 importe) ; celles des contacts et des écritures, par **aucun** test
   d'API (`grep -rnE 'limit=0|limit=1000|offset=-' crates/kesh-api/tests` → postes ouverts et réconciliation seuls).
4. **Validation de la liste des écritures** (`journal_entries.rs:355-439`), dans cet ordre : `dateFrom` puis `dateTo`
   (analyse), `amountMin` puis `amountMax` (analyse), `amountMin` puis `amountMax` négatifs, `amountMin > amountMax`,
   `dateFrom > dateTo`, `accountId <= 0`, puis `description` rognée et vidée si blanche (l. 432-436). Neuf refus
   `AppError::Validation` aux messages écrits en dur ; **aucun** n'est figé par un test d'API (`grep -rnE "dateFrom
   invalide|amountMin|accountId doit" crates/kesh-api/tests` → rien). La liste ne lit **aucune** société : elle passe
   `current_user.company_id` au dépôt (l. 456).
5. **Le détail d'une écriture** (`journal_entries.rs:489-532`) : `get_company_for` (l. 494), `find_by_id(&state.pool,
   company.id, id)` → 404 si absent (l. 496-498), puis **une** transaction de lecture (l. 506) pour quatre lecteurs —
   `reversal_blocker`, `reversed_by`, `modification_blocker`, `imported_supplier_invoices::find_source_document_of_entry`
   — et `rollback` (l. 514). Une garde **lexicale** fige cette forme sur le corps du handler :
   `get_journal_entry_reads_in_one_transaction` (`mod tests`, l. 1005-1080 ; `include_str!("journal_entries.rs")`
   l. 1016, `find("pub async fn get_journal_entry(")` l. 1023, `corps.contains("journal_entries::find_by_id(&state.pool")`
   l. 1077). Déplacer le corps la ferait rougir (C-32-4a-7).
6. **Aucune des cinq lectures n'écrit** : aucun `audit`, `insert_in_tx` ni `get_or_create` dans leurs corps (`grep -n
   'get_or_create\|audit' routes/{contacts,letterings,journal_entries}.rs` → commentaires seuls). **Fermeture des appels**
   (relevée le 2026-10-24 en suivant chaque appel jusqu'à la requête, validation P1, F1 ; chemins `kesh-db/src/repositories/`) :
   - **appels directs** des handlers : `journal_entries::{list_by_company_paginated, find_by_id, reversal_blocker,
     reversed_by, modification_blocker}`, `imported_supplier_invoices::find_source_document_of_entry` (« Lecture
     **ordinaire** », doc-comment `imported_supplier_invoices.rs:373-378` ; sa requête, l. 386-395, est un `SELECT … ORDER BY
     si.id LIMIT 1` sans verrou), `letterings::open_items`,
     `contacts::{list_by_company_paginated, find_by_id_in_company}`, `helpers::company_for_ids` → `companies::find_by_id`
     (`FIND_BY_ID_SQL`, `companies.rs:24-27`) ;
   - **appels transitifs** : `reversal_blocker` → `reversal_blockers` (l. 2368) → `document_owners` (l. 2212) ;
     `modification_blocker` (l. 1215) → `fiscal_years::find_later_closed` (`FIND_LATER_CLOSED_SQL`, sans verrou ; la
     variante verrouillante est `find_later_closed_in_tx`, l. 870, hors chemin), `modification_guard` (l. 1066 →
     `reversal_blockers`, puis un `SELECT` sur `supplier_invoices`/`audit_log`), un `SELECT books_locked_through`, et
     `lettering_guard(…, Lecture::Conseil)` (l. 1261) ; `open_items` (`open_items.rs:346`, sa transaction `conn.begin()`
     l. 354, refermée par `rollback` l. 405) → `require_viewable_account` → `letterings::letterable_account`
     (`letterings.rs:376`), les trois `const` `OPEN_ITEMS_*_SQL`, `lines_documents_and_periods` (`letterings.rs:945` →
     `document_owners`, `open_period_rule` → `find_later_closed` et `books_locked_through`), `invoice_states`
     (`open_items.rs:476`) ;
   - **aides pures et textes SQL lus par la fermeture** (validation P2, R2 = F2) : `push_where_clauses`
     (`journal_entries.rs:849`, `contacts.rs:150`, appelée par les deux `list_by_company_paginated`),
     `util::search::{escape_boolean_ft, escape_like}` (`kesh-db/src/util/search.rs`), `letterings::code_of`
     (`letterings.rs:333`, par `lettering_guard`), `with_placeholders` (`letterings.rs:315`), `client_invoice_of` et
     `invoice_states_sql()` (`open_items.rs:310`, `:120`) ; les `const` `LETTERING_GUARD_SQL` (`journal_entries.rs:1146`),
     `ENTRY_COLUMNS`/`LINE_COLUMNS` (l. 100, 103), `BANK_LINKED_SQL` (`letterings.rs:353`),
     `OPEN_PERIOD_RULE_FISCAL_YEARS_SQL` (l. 277), `FIND_LATER_CLOSED_SQL` (`fiscal_years.rs:890`), `COLUMNS` et
     `FIND_BY_ID_SQL` de `contacts.rs` (l. 33, 42) ; et le premier `SELECT` de `modification_blocker` (`fiscal_years`
     joint, `journal_entries.rs:1222-1229`). La fermeture se définit ainsi : **toute fonction ou `const` atteinte depuis
     un appel direct**, la liste ci-dessus en étant le relevé du 2026-10-24.
   Sur cette fermeture, **aucune** écriture ni `get_or_create` ; **un seul** `FOR UPDATE` : la branche
   `Lecture::Verrouillante` de `lettering_guard` (`journal_entries.rs:1182`), **non prise** — `modification_blocker`
   passe `Lecture::Conseil` (l. 1261 ; « Lecture **sans verrou** », l. 1205). Les autres `FOR UPDATE` de ces fichiers
   sont dans des fonctions hors de la fermeture (`create_in_tx_inner`, `update_in_tx`, `delete_in_tx`,
   `reverse_in_tx_inner`, `find_later_closed_in_tx`, `lock_fiscal_years_of_group`…). **Établi par lecture statique.**
   L'exécution (AC11) le confirme **pour les écritures** sur les branches que les fixtures exercent ; **pour les
   verrous**, elle ne le confirme que si une session `SET SESSION TRANSACTION READ ONLY` refuse aussi un `SELECT … FOR
   UPDATE` — ce que le dépôt a **mesuré** pour une transaction ouverte par `START TRANSACTION … READ ONLY` (MariaDB
   10.11.16, erreur 1792, SQLSTATE `25006` : test `t0_un_verrou_dans_l_instantane_est_refuse_par_le_serveur`,
   `kesh-db/tests/closing_checks.rs:253-274`, Story 29-1), **non** pour la forme de session qu'emploie l'aide
   `lecture_seule` : **non établi**, mesuré par l'AC11 (c).
7. **Ordres de lecture** : `get_contact` lit le contact (`find_by_id_in_company`, l. 613-615) **avant** la société
   (l. 618) ; `list_contacts` lit la société d'abord (l. 570) ; `get_open_items` analyse `asOf` (l. 540, `parse_as_of`
   l. 512-527, `Utc::now()` si absent) **avant** de lire la société (l. 543). La société des contacts ne sert qu'à la
   **présentation** — le libellé des conditions de paiement, `contact_response_with_label` (l. 510-522), par
   `state.i18n`.
8. **Le motif de la 32-3, tel qu'il est** : `ContexteAcces`, `Exigence`, `Portee { BooksRead, InvoicesRead }`
   (`services/acces.rs`) ; `Lecture`/`Trace` (`services/poignees.rs`), `pool()` en `pub(in crate::services)` ;
   `AppState::lecture()`/`trace()` (`lib.rs:152-163`) ; `#![cfg_attr(not(test), forbid(unsafe_code))]` en tête de
   `lib.rs` **et** de `services/mod.rs` (l. 100, C-32-3-36) ; le pilote simple `plan_comptable::lister` (exigence
   constante `EXIGENCE_LISTER`, `company_for_ids` d'abord). La garde de `tests/services_motif.rs` : (ii) sur
   `list_accounts` seul (l. 2073-2100, **sans témoin positif** — #738 A5) ; `services/` compté à **cinq** fichiers
   (l. 2137-2142) ; `TOUTES` à deux portées (l. 869) ; `autre_portee` en `match` exhaustif sur deux variantes
   (l. 894-899) ; `pool_lecture_seule` (l. 835-851).
9. **`docs/MULTI-TENANT-SCOPING-PATTERNS.md`** écrit « Every handler validates company exists » (l. 273), avec la note
   de la 32-3 (l. 273-276, `GET /api/v1/accounts`) ; la liste des écritures fait exception (fait 4), aujourd'hui
   comme demain. **`docs/api-external.md`** décrit ces routes (l. 213, 216-218) et les bornes des postes ouverts et des
   propositions (l. 405 : « `limit` (défaut 50, ramené entre 1 et 500), `offset` (défaut 0, ramené à 0 s'il est
   négatif) » ; l. 441) — rien sur celles des contacts ni des écritures.

## Le rollout (ce que la story déplace)

| Handler (l.) | Route | Service | Exigence API / MCP | Demande | Rend |
|---|---|---|---|---|---|
| `list_journal_entries` (345) | `GET /journal-entries` | `ecritures::lister` | Consultation / BooksRead | `DemandeListeEcritures` (champs du DTO, chaînes brutes, `limit`/`offset` en `Option`) | `JournalEntryListResult` |
| `get_journal_entry` (489) | `GET /journal-entries/{id}` | `ecritures::detail` | Consultation / BooksRead | `id: i64` | `DetailEcriture` |
| `get_open_items` (534) | `GET /accounts/{id}/open-items` | `postes_ouverts::lister` | Consultation / BooksRead | `DemandePostesOuverts { account_id, as_of: Option<String>, limit, offset }` | `OpenItems` |
| `list_contacts` (563) | `GET /contacts` | `carnet::lister` | Consultation / **ContactsRead** | `DemandeListeContacts` (champs du DTO) | `PageContacts { page, societe }` |
| `get_contact` (607) | `GET /contacts/{id}` | `carnet::detail` | Consultation / **ContactsRead** | `id: i64` | `FicheContact { contact, societe }` |

**Restent dans leur handler**, et sont figés comme tels (AC12 (iii)) : `get_lettering`, `get_lettering_proposals`
(aucun outil ne les sert, C-32-4a-1), et les **dix** mutants (C-32-0-4).

## Critères d'acceptation

### A — Les modules et leurs types

1. **AC1 — trois modules de services.** `crates/kesh-api/src/services/{ecritures,postes_ouverts,carnet}.rs` neufs
   (C-32-4a-3), déclarés `pub mod` dans `services/mod.rs` (aux côtés des quatre de la 32-3, qui ne bougent pas), sous
   son `forbid(unsafe_code)`, qu'ils ne redéclarent ni ne déplacent (R8). Chaque élément public porte son doc-comment.
   **Aucun type d'Axum, `AppState` ni `CurrentUser`** dans leurs lignes de code (R1) — la règle de l'AC1 de la 32-3,
   exécutée par la garde (vi) de `services_motif.rs`, dont la liste `RESIDUS_VI` **ne change pas**. **Aucune `Trace`** :
   ces services ne tracent rien (fait 6) ; leurs lignes de code ne contiennent ni `\bTrace\b` ni `\btrace\b` (garde
   (viii), AC12). `services/acces.rs` gagne `Portee::ContactsRead` (« lecture du carnet d'adresses : contacts, données
   personnelles »), sans chaîne ni persistance (C-32-4a-4). Le doc-comment de `services/mod.rs` gagne : la liste des
   services du rollout et leurs exigences, l'inventaire de l'AC13 (son point 3 étendu), et les deux lectures non
   migrées des fichiers visés avec leur raison (C-32-4a-1).
2. **AC2 — le contrôle d'accès en tête.** Chacun des cinq services commence par `acces.exiger(EXIGENCE_…)?` (R2),
   **avant** l'analyse de la demande et toute lecture ; une constante `pub const EXIGENCE_<OPÉRATION>: Exigence` par
   service, à côté de lui (R3), aux valeurs du tableau du *Rollout*. Le rôle API **égale** celui de la route (fait 2) ;
   le middleware refuse toujours le premier.

### B — Les services

3. **AC3 — `ecritures::lister`.** Reçoit `(&ContexteAcces, &Lecture, DemandeListeEcritures)`. Après `exiger` : défaut
   (`LIMITE_DEFAUT` 50) puis écrêtage `clamp(1, LIMITE_MAX)` (500) de `limit`, `offset` défaut 0 puis `max(0)` ; puis
   les **neuf** contrôles du fait 4, **dans cet ordre, messages identiques au caractère près** (noms de paramètres en
   camelCase compris, C-32-4a-5) ; puis la description rognée et vidée si blanche ; puis
   `journal_entries::list_by_company_paginated(lecture.pool(), acces.company_id(), query)`. **Aucune lecture de
   société** (fait 4 — comportement conservé, C-32-4a-6). La conversion `CoreJournal → DbJournal` se fait au service ou
   au handler (au développement, à écrire au Dev Agent Record) sans changer l'ordre des erreurs : une valeur de
   `journal` invalide reste un refus de l'extracteur `Query` d'Axum, avant le service.
4. **AC4 — `ecritures::detail`.** Reçoit `(&ContexteAcces, &Lecture, id)`. Après `exiger` : `company_for_ids`
   (`Internal` si la société manque), `find_by_id(lecture.pool(), company.id, id)`
   (`AppError::Database(DbError::NotFound)` → 404, la forme d'aujourd'hui, `journal_entries.rs:498`), puis **une**
   transaction `lecture.pool().begin()` portant les quatre lecteurs du fait 5, et `rollback`. Rend
   `DetailEcriture` (champs `pub` : l'écriture et ses lignes, l'écriture qui la contre-passe, le motif de blocage de la
   contre-passation, le motif de blocage de la modification, la pièce source — noms au développement). La garde du fait 5
   **se déplace** au `#[cfg(test)] mod tests` de `services/ecritures.rs` (C-32-4a-7) : même nom
   (`get_journal_entry_reads_in_one_transaction`, ou `detail_lit_dans_une_transaction` avec renvoi à la 15-1b-0 au
   doc-comment — au choix, écrit au Dev Agent Record), mêmes assertions sur le corps de `pub async fn detail(` lu par
   `include_str!("ecritures.rs")` tronqué à `#[cfg(test)]` : une transaction ouverte une fois, `acquire()` zéro fois,
   chacun des quatre lecteurs appelé une fois sur `&mut tx` ou `&mut *tx` et non sur le pool, et l'écart assumé.
   **Trois jetons changent** (validation P1, R3 = F3), et chacun s'écrit tel quel : (1) le compte de la transaction —
   `pool.begin()` (`journal_entries.rs:1030`), que `lecture.pool().begin()` ne contient pas (`pool().begin()`) et qui y
   compterait **zéro** — devient `pool().begin()`, compté sur le corps **débarrassé de ses blancs** (`chars().filter(|c|
   !c.is_whitespace())`), insensible à une coupure de rustfmt (`lecture\n.pool()\n.begin()`) ; (2) l'écart assumé
   `journal_entries::find_by_id(&state.pool` (l. 1077) devient `journal_entries::find_by_id(lecture.pool()` — le
   paramètre du service s'appelle donc `lecture` (et le contexte `acces`, comme `plan_comptable::lister`,
   `services/plan_comptable.rs:35-36`), le jeton en dépend (validation P2, R4) ; (3) le refus
   d'un lecteur sur le pool (`!appels[0].contains("state.pool")`, l. 1065) cherche `.pool()`. S'y ajoutent les deux
   ancres (`include_str!("ecritures.rs")`, `find("pub async fn detail(")`). Corriger un rouge de cette garde en
   **relâchant** un motif est interdit : le diff entre l'ancienne et la nouvelle garde est recopié au Dev Agent Record,
   jeton par jeton.
5. **AC5 — `postes_ouverts::lister`.** Reçoit `(&ContexteAcces, &Lecture, DemandePostesOuverts)`. Après `exiger` :
   `parse_as_of` (déplacée telle quelle de `letterings.rs:504-527`, doc-comment compris, `Utc::now().naive_utc().date()`
   si absente ou vide), puis défaut (`LIMITE_DEFAUT` 50) et `clamp(1, LIMITE_MAX)`, `offset` défaut 0 puis `max(0)`,
   puis `company_for_ids`, puis une connexion acquise de la `Lecture` et `letterings::open_items(&mut conn, company.id,
   account_id, as_of, limit, offset)`. **Ordre des refus inchangé** (AC7 de la 15-1b, doc-comment l. 530-533) : `asOf`
   (400) avant toute lecture, compte introuvable ou d'une autre société (404), compte non lettrable (409).
   `LIMITE_MAX` est **le plafond commun** : `get_lettering_proposals`, qui reste au handler, l'importe du service au lieu
   de son `MAX_LIMIT` local, qui disparaît (une seule constante, C-32-4a-5) ; `PROPOSALS_DEFAULT_LIMIT` reste au handler.
6. **AC6 — `carnet::lister` et `carnet::detail`.** `lister(&ContexteAcces, &Lecture, DemandeListeContacts)` : après
   `exiger`, `company_for_ids` **d'abord** (fait 7), puis défaut (`LIMITE_DEFAUT` 20) et `clamp(1, LIMITE_MAX)` (100),
   `offset` défaut 0 puis `max(0)`, et les **défauts du tri** d'aujourd'hui (`sort_by` : `unwrap_or_default()`,
   `sort_direction` : `unwrap_or(SortDirection::Asc)`, `contacts.rs:584-585` — le MCP les reçoit en `None` ; validation
   P2, R8), puis `contacts::list_by_company_paginated(lecture.pool(), acces.company_id(), query)` ; rend `PageContacts { page, societe }`. `detail(&ContexteAcces, &Lecture, id)` : après `exiger`,
   `find_by_id_in_company(lecture.pool(), id, acces.company_id())` → `AppError::Database(DbError::NotFound)`
   (`contacts.rs:615`) **avant** `company_for_ids` (fait 7) ;
   rend `FicheContact { contact, societe }`. Le libellé des conditions de paiement reste au handler
   (`contact_response_with_label`, C-32-4a-8).
7. **AC7 — handlers minces, comportement inchangé.** Chacun des cinq handlers ne fait plus que : extraire la requête,
   construire `ContexteAcces::depuis_api(&current_user)` et la demande, appeler son service avec `&state.lecture()`,
   mettre en forme la réponse (`ListResponse`, `JournalEntryDetailResponse`, `OpenItemsResponse`,
   `contact_response_with_label`). Aucun d'eux ne lit plus `state.pool` (AC12 (ii)). Statuts, corps, en-têtes, ordre des
   erreurs : identiques. Les DTO (`ListJournalEntriesQuery`, `OpenItemsQuery`, `ListContactsQuery`) restent au handler ;
   leurs défauts serde (`default_limit`, `default_open_items_limit`) rendent les constantes **du service** ; les
   constantes locales qu'elles remplacent disparaissent (`DEFAULT_LIMIT`/`MAX_LIMIT` de `journal_entries.rs:41-42`,
   `OPEN_ITEMS_DEFAULT_LIMIT`/`MAX_LIMIT` de `letterings.rs:288, 292`, `MAX_LIST_LIMIT`/`DEFAULT_LIST_LIMIT` de
   `contacts.rs:42-43`) — `grep -nE "\b(MAX_LIMIT|DEFAULT_LIMIT|MAX_LIST_LIMIT|DEFAULT_LIST_LIMIT|OPEN_ITEMS_DEFAULT_LIMIT)\b"
   crates/kesh-api/src/routes/{journal_entries,letterings,contacts}.rs` ne rend plus que leurs **usages** par chemin
   du service (ou rien), recopié au Dev Agent Record. Ce contrôle ne voit que la disparition des **noms** — une borne
   recopiée sous un autre nom (`const PAGE_MAX: i64 = 500` au handler) lui échappe (validation P1, F7) ; d'où le second,
   sur les **écrêtages** : `grep -nE 'clamp\(|\.max\(0\)' crates/kesh-api/src/routes/{journal_entries,letterings,contacts}.rs`
   rend aujourd'hui huit lignes (`contacts.rs:575-576`, `letterings.rs:541-542`, `:563-564`, `journal_entries.rs:352-353`)
   et ne doit plus rendre que **deux** lignes, toutes deux dans `get_lettering_proposals` : le commentaire
   `` `clamp(1, 500)` `` et l'écrêtage `params.limit.clamp(1, <plafond du service>) as usize` (numéros de `89d68704` :
   563-564 ; le correctif les déplace — validation P2, F6), sortie recopiée au Dev Agent Record. Les mutants, `get_lettering` et
   `get_lettering_proposals` ne changent pas, hors l'import du plafond commun (AC5).

### C — Les tests

8. **AC8 — les tests existants restent verts ; les seules modifications sont celles-ci.** (1) `tests/services_motif.rs` :
   `TOUTES` à trois portées (l. 869) et un bras `ContactsRead` à `autre_portee` (l. 894-899) (C-32-4a-4) ; le compte de
   `services/` à **huit** fichiers (l. 2137-2142) ; la garde étendue de l'AC12 ; `mod common;` ajouté en tête (le
   fichier ne le déclare pas aujourd'hui — `grep -n '^mod common' tests/services_motif.rs` ne rend rien ; validation P1,
   R5 = F4 —, et `common/mod.rs`, monté par 23 binaires, se compile sous son `#![allow(dead_code)]`) ;
   `pool_lecture_seule` et `lecture_seule` (l. 833-855) remplacées par l'aide commune ; **une dix-neuvième ligne** à
   `TABLE_DES_TRACES`, `journaux_affichage_sans_filtre` (`num: 19`, `Svc::Journaux`, `Usg::Affichage`, `dem: BORNES`,
   `"journal_filter": null`), la **ligne 7 restant filtrée** (`Some("Achats")`), et les sites qui comptent la table en
   conséquence — l'assertion `TABLE_DES_TRACES.len()` (l. 773-777 : « dix-neuf lignes pour dix-huit couples (service,
   usage) ») et le doc-comment de `table_des_traces_par_la_route` (l. 765) ; celui du module (l. 4, « chacun des
   dix-huit handlers ») reste vrai (AC17, C-32-4a-10). (2) `tests/common/mod.rs` : `pub mod lecture_seule;` ;
   `tests/common/lecture_seule.rs` neuf (C-32-4a-9). (3) `routes/journal_entries.rs` : le `mod tests` perd la seule garde du fait 5 (l. 1005-1080),
   déplacée (AC4) ; ses autres tests sont identiques. Contrôles au dernier commit :
   `git diff --diff-filter=MDR --name-only $(git merge-base HEAD origin/main) HEAD -- crates/kesh-api/tests` → **exactement**
   `crates/kesh-api/tests/common/mod.rs` et `crates/kesh-api/tests/services_motif.rs` ;
   `diff <(git show $(git merge-base HEAD origin/main):crates/kesh-api/src/routes/contacts.rs | sed -n '/^#\[cfg(test)\]/,$p') <(sed -n '/^#\[cfg(test)\]/,$p' crates/kesh-api/src/routes/contacts.rs)`
   → **vide** ; le même `diff` sur `journal_entries.rs` → la **seule** suppression du bloc de la garde. Fichiers qui
   doivent rester verts, relevé **large** du 2026-10-24 (`grep -lE '/api/v1/journal-entries|/api/v1/contacts|open-items|lettering-proposals|/api/v1/letterings' crates/kesh-api/tests/*.rs`
   → **16** fichiers, `sqlx::test` par `grep -c`) : `admin_full_import_e2e` 56, `contact_client_number_e2e` 8,
   `contact_payment_terms_e2e` 9, `contact_persons_e2e` 4, `filet_bilan_clos_e2e` 2, `idor_multi_tenant_e2e` 14,
   `inbox_import_e2e` 40, `invoice_ttc_e2e` 3, `journal_entry_reversal_e2e` 56, `kf004_no_op_e2e` 6, `letterings_e2e` 11,
   `onboarding_e2e` 16, `opening_balances_e2e` 31, `open_items_e2e` 7, `period_lock_e2e` 16, `reports_e2e` 39 — dont
   le développement relève, au site, ceux qui frappent l'une des **cinq** routes en `GET` (au moins
   `contact_client_number_e2e`, `contact_payment_terms_e2e`, `idor_multi_tenant_e2e`, `journal_entry_reversal_e2e`,
   `letterings_e2e`, `open_items_e2e`), et recopie au Dev Agent Record la liste et les comptes **réellement exécutés**.
   S'y ajoutent les huit **gardes de sources** de l'AC7 de la 32-3 (`audit_label_registry`, `textes_coherents` — G4-ter :
   aucun doc-comment neuf ne cite un compte sous la forme « NNNN Nom » —, `admin_pat_denied_e2e`, `audit_route_registry`,
   `kesh-db/tests/test_schema_guard.rs` — chaque `#[sqlx::test(` de `services_rollout.rs` tient sur une ligne et nomme
   `../kesh-db/test-schema` —, `configuration_transmise`, `bank_account_closed_inventory`,
   `kesh-db/tests/letterings_lexical.rs`), relancées par le gate ciblé.
9. **AC9 — un test de refus par service, et son contrôle positif.** Fichier neuf `crates/kesh-api/tests/services_rollout.rs`
   (`#[sqlx::test(migrations = "../kesh-db/test-schema")]`, fixtures **synthétiques** : `seed_accounting_company`, une
   écriture sur un compte lettrable — 1000 et 2000 le sont, aucun compte bancaire n'y étant lié dans le seed —, un
   contact). Ces fixtures s'écrivent dans `services_rollout.rs` : `poser_livres` est privée à `services_motif.rs`. Le
   montage HTTP (`spawn_app`, `forge_jwt`) et les constructeurs de contexte (`mcp_societe`, `api_societe`, `INEXISTANT`)
   s'y **recopient** de `services_motif.rs:43-141` (constantes, `TestApp`, `test_config`, `spawn_app`, `forge_jwt` ;
   validation P2, F4) et `:830-883` — recopie assumée, convention du dépôt (`spawn_app` est
   recopiée dans 68 binaires de test, `forge_jwt` dans 29 ; validation P1, F4) ; la 32-4b ajoutera ses tests à ce même
   fichier, sans troisième copie. **Erreurs attendues, à la lettre** (validation P1, R4 = F5) : « `NotFound` » s'écrit
   `AppError::Database(DbError::NotFound)` (la variante `AppError::NotFound` n'existe pas, `errors.rs:138-170`) ; le
   compte non lettrable, `AppError::Database(DbError::LetteringAccountNotLetterable)` ; `AppError` n'étant pas
   `PartialEq` (`#[derive(Debug, Error)]`), les assertions s'écrivent en `matches!`. Une assertion nommée par cas
   (C-32-3-38), pour **chacun des cinq services** :
   - (a) canal **MCP sans la portée** (portées vides ; puis les **deux autres** portées seules) → `Err(AppError::Forbidden)` ;
   - (b) contrôle positif : MCP **avec** la portée exigée → `Ok` ;
   - (c) **le refus précède la validation et la lecture** — MCP sans portée, demande qui échouerait sans contrôle →
     `Forbidden`, et le **même cas avec la portée** rend l'erreur attendue (témoin du cas) :
     - (c-val) `dateFrom = "x"` pour `ecritures::lister` (attendu sans contrôle : `Validation("dateFrom invalide (…)")`) ;
       `as_of = "2026-13-01"` pour `postes_ouverts::lister` (`Validation`) ;
     - (c-lec) `id` inexistant pour `ecritures::detail` et `carnet::detail` (`NotFound`) ; compte inexistant pour
       `postes_ouverts::lister` (`NotFound`) ; société inexistante pour `ecritures::detail`, `postes_ouverts::lister` et
       `carnet::lister` (`Internal`) ;
     - sans objet : (c-lec) pour `ecritures::lister` (aucune entité nommée par la demande n'est lue avant la page) et
       (c-val) pour `ecritures::detail`, `carnet::lister`, `carnet::detail` (leur demande ne porte rien qui échoue à
       l'analyse) — dit au Dev Agent Record ;
   - (e) canal **API** `Consultation` → `Ok` pour les cinq.
   **Et l'ordre de `carnet::detail`** : MCP avec `ContactsRead`, société inexistante **et** contact inexistant →
   `NotFound` (le contact est lu d'abord, fait 7), quand `carnet::lister` sur la même société inexistante rend `Internal`.
10. **AC10 — la validation et les bornes, au service et par la route.** Dans `services_rollout.rs` :
    - (a) **table des neuf refus** de `ecritures::lister` (fait 4), une ligne par refus, message attendu **en entier**
      (le suffixe d'analyse `(…)` des quatre erreurs d'analyse comparé par préfixe jusqu'à la parenthèse ouvrante
      incluse), et un cas **d'ordre** : `dateFrom = "x"` **et** `accountId = 0` → le message de `dateFrom` ; plus un cas
      `description = "   "` → aucune restriction (le nombre d'écritures égale celui sans filtre) ;
    - (b) **table des bornes** des trois services paginés, au service : `(limit, offset)` demandés → rendus (`limit`,
      `offset` de la page) : `(None, None)` → `(défaut, 0)` ; `(Some(0), Some(-3))` → `(1, 0)` ;
      `(Some(10_000), Some(0))` → `(plafond, 0)` — défauts et plafonds : écritures 50 / 500, postes ouverts 50 / 500,
      contacts 20 / 100 ;
    - (c) **par la route** (montage HTTP de l'AC9, `services_motif.rs:43-141`) : `GET /api/v1/contacts?limit=1000&offset=-3`
      → `limit` 100, `offset` 0 ; `GET /api/v1/contacts` → `limit` 20 ; `GET /api/v1/journal-entries?limit=0` →
      `limit` 1 ; `GET /api/v1/journal-entries` → `limit` 50 ; `GET /api/v1/journal-entries?dateFrom=x` → 400, code
      `VALIDATION_ERROR` et message de la table. Les bornes des postes ouverts par la route sont déjà figées
      (`open_items_e2e.rs:274-323` ; `:529-539` pour les propositions, le plafond commun de l'AC5), non recopiées.
11. **AC11 — lecture seule, et filtre de société.** Avec l'aide commune `lecture_seule` (pool dont chaque connexion
    passe `SET SESSION TRANSACTION READ ONLY`, C-32-4a-9) :
    - (a) les **cinq** services **réussissent** sur cette `Lecture` (le détail d'écriture compris : `BEGIN`, quatre
      lectures, `ROLLBACK` ; les postes ouverts : leur transaction propre) — confirmation, **sur les branches que les
      fixtures exercent**, de l'inventaire de l'AC13 — **pour les écritures** ; pour les verrous, seulement si l'AC11 (c)
      établit le refus ; les branches non atteintes (pièce source présente, motifs de blocage présents…) sont
      **nommées** au Dev Agent Record, non revendiquées ;
    - (b) **filtre de société** : une seconde société, posée par `kesh_db::repositories::companies::create` (comme
      `common::create_test_company`) — **non** par un second `seed_accounting_company`, qui échouerait sur l'unicité de
      `users.username` (il insère `admin` et `changeme`, `kesh-db/src/test_fixtures.rs:96-114` ; `uq_users_username`,
      `test-schema/0001_schema_squash.sql:1143` ; validation P1, R6) ; un contexte MCP de cette société, toutes portées,
      demande le détail de l'écriture et du contact de la première → `AppError::Database(DbError::NotFound)` ; les postes
      ouverts du compte de la première → idem ; `ecritures::lister` et `carnet::lister` → aucune ligne de la première ;
    - (c) **mesure, puis témoin : un verrou sur la `Lecture` de test** (validation P1, F1) — au patron de la T0 de la
      29-1 (`kesh-db/tests/closing_checks.rs:253-274`) : le développement **mesure** d'abord ce que MariaDB fait de
      `SELECT 1 FROM fiscal_years LIMIT 1 FOR UPDATE` sur `pool_lecture_seule` (session `SET SESSION TRANSACTION READ
      ONLY`), **en autocommit puis dans `begin()`**, et recopie l'issue au Dev Agent Record ; puis il la **fige** par un
      test qui l'asserte (erreur 1792 attendue d'après la 29-1, non établie pour la forme de session). Si les deux sont
      refusés, l'AC11 (a) confirme aussi l'absence de verrou sur les branches exercées, et le doc-comment de
      `services/mod.rs` peut le dire ; si l'un est accepté, l'absence de `FOR UPDATE` reste établie **par la seule
      lecture statique** (fait 6, AC13), et le doc-comment comme le Dev Agent Record l'écrivent ainsi.
12. **AC12 — la garde lexicale, étendue (R8), dans `services_motif.rs`.** **Lignes de code** : la définition de l'AC10
    de la 32-3. **Corps d'un handler** — une seule définition pour (ii) et (iii), celle de `corps_des_handlers`
    (`services_motif.rs:2009-2033`) : de la ligne qui commence par `pub async fn <nom>` à la **première ligne égale à
    `}`**, sur le texte du fichier tronqué à son `#[cfg(test)]` ; chaque corps se choisit par son **nom exact** (le
    préfixe `get_lettering` rend aussi `get_lettering_proposals`). La (ii) de la 32-3, qui lit `list_accounts` jusqu'au
    `pub async fn` suivant (l. 2088-2090), s'y ramène : entre les deux, `accounts.rs` ne porte que des doc-comments.
    (Validation P2, R3 : la P1 énonçait « jusqu'au `pub async fn` suivant » et raisonnait sur `corps_des_handlers`.)
    La garde de transaction de l'AC4 garde, elle, sa propre délimitation, déplacée telle quelle. **Détecteur commun à
    (ii) et (iii)** (validation P1, R2 = F2) : non plus `contains("state.pool")` ligne à ligne (le
    patron de `ac10_i_ii_handlers_sans_le_pool`, `services_motif.rs:2073-2100`), mais le motif `\bstate\s*\.\s*pool\b`
    sur le corps **aplati** — `sans_commentaires` (l. 1998) appliquée au corps joint par `\n`, `\s` traversant les fins
    de ligne. Raison : rustfmt coupe `state.pool.acquire()…` en `state` / `.pool` sur deux lignes, forme présente dans
    les fichiers mêmes que la garde lit (`letterings.rs:243-244`, `:544-545`, `:566-567` ; `grep -nE '^\s*\.pool\b'`),
    et qu'un détecteur ligne à ligne ne voit pas (#738 B4). Aucun faux vert aujourd'hui — chacun des douze handlers
    attendus porte aussi une occurrence non coupée —, mais un handler neuf écrit comme ses voisins y échapperait ;
    - (ii) étendue : les corps des **six** handlers `accounts::list_accounts`, `journal_entries::{list_journal_entries,
      get_journal_entry}`, `letterings::get_open_items`, `contacts::{list_contacts, get_contact}` ne contiennent pas
      `state.pool` ; chaque corps est non vide (plancher de lignes) ; **témoins positifs** (#738 A5) : le même détecteur
      trouve `state.pool` dans un texte témoin **en une ligne**, dans un texte témoin **coupé**
      (`"let mut conn = state\n        .pool\n        .acquire()"`), et dans le corps réel de `letterings::get_lettering` ;
      il ne le trouve pas dans `// state.pool` ni dans `lecture.pool()` ;
    - (iii) **inventaire des sites non résolus** : dans les trois fichiers visés, l'ensemble des couples (fichier,
      handler) dont le corps contient `state.pool` est **exactement** les douze attendus — les dix mutants
      (`journal_entries::{reverse_journal_entry, reverse_journal_entry_at_next_fiscal_year_start, create_journal_entry,
      update_journal_entry, delete_journal_entry}`, `letterings::{create_lettering, delete_lettering}`,
      `contacts::{create_contact, update_contact, archive_contact}`) et `letterings::{get_lettering,
      get_lettering_proposals}` (C-32-4a-1) — et le nombre de `pub async fn` par fichier vaut 7 / 5 / 5 ; **et, au
      niveau du fichier**, aucune occurrence hors d'un corps : sur le code de chacun des trois fichiers entier (tronqué
      au `#[cfg(test)]`, `sans_commentaires`, aplati), le nombre d'occurrences du détecteur **égale** la somme de ses
      occurrences sur les corps — aujourd'hui 15 / 10 / 12 des deux côtés (relevé du 2026-10-24 ; aucun `/*` dans ces
      fichiers — un bloc qui enjamberait deux corps ferait diverger les deux comptes : faux rouge seulement). Ce compte voit ce qu'aucun corps ne lit : une fonction asynchrone non `pub` (`pub(crate) async fn`,
      `async fn`), que `corps_des_handlers` n'ouvre pas, et une **aide synchrone** posée entre deux handlers
      (`fn conn(state: &AppState) -> … { state.pool.acquire() }`) qu'appellerait un handler migré (validation P2, R3) ;
      il remplace le compte des `async fn` toutes visibilités écrit à la validation P1. Il ne vaut que pour ces trois
      fichiers : `accounts.rs` porte légitimement `state.pool` hors de tout corps (l'aide `response`, l. 185-190, des
      mutants), et (ii) n'y lit que le corps de `list_accounts`. Un handler neuf sur `state.pool`, en une ligne ou coupé,
      rougit par son nom ou par le compte du fichier. **Les limites qui restent** (#738 B4, hors périmètre) : le champ
      sous un autre nom que `state` — une déstructuration `State(AppState { pool, .. })`, une liaison renommée
      (`s: &AppState` puis `s.pool`, `let st = &state;` puis `st.pool`) —, dans un handler comme dans une aide ; et, dans
      `accounts.rs`, une aide appelée par `list_accounts` ;
    - (vi) inchangée (`RESIDUS_VI`), le compte de `services/` passant à **huit** fichiers exactement ;
    - (viii) **neuve** : les lignes de code de `services/{ecritures,postes_ouverts,carnet}.rs` ne contiennent ni
      `\bTrace\b` ni `\btrace\b` (témoin positif : le motif trouve `trace: &Trace` dans un texte témoin, et ne trouve pas
      `tracing::warn!`) ;
    - (i), (v-1), (v-2), (vii) **inchangées** — la v-2 continue de compter **deux** fabriques au câblage de `lib.rs`
      (les handlers migrés appellent `state.lecture()`, jamais `Lecture::new(`).
    Ce que ces gardes ne voient pas est celui de la 32-3 (AC10, *Ce que les gardes ne voient pas*) : la revue lit chaque
    `.pool()` des trois fichiers neufs.

### D — Inventaire, mutations, gates, documentation

13. **AC13 — inventaire clos des écritures sur les cinq chemins.** Écrit au doc-comment de `services/mod.rs` (point 3
    étendu) et au Dev Agent Record : (1) **aucune** écriture des services, **sur la fermeture des appels** du fait 6
    (appels directs **et** transitifs, `const` SQL comprises — non sur le seul corps des fonctions nommées : `reversal_blocker`
    n'est qu'un `reversal_blockers(…).next()`, et les requêtes vivent souvent dans des `const` hors du corps, validation
    P1, F1) — établi par lecture statique : le développement relance, sur le **code** (commentaires écartés, même
    définition qu'à l'AC12 ; validation P1, R9), `grep -nE "INSERT|UPDATE |DELETE|FOR UPDATE|get_or_create"` sur les
    **fichiers** de la fermeture (`repositories/{journal_entries,fiscal_years,imported_supplier_invoices,letterings,
    letterings/open_items,contacts,companies}.rs` et `util/search.rs`), **chacun tronqué à son `#[cfg(test)]`** (`journal_entries.rs:3111`,
    `fiscal_years.rs:1394`, `letterings.rs:2215`, `open_items.rs:618`, `contacts.rs:738`, `search.rs:158` ; aucun dans
    les deux autres) — les `INSERT` des tests ne se trient pas (validation P2, R2 = F2), range chaque
    occurrence sous sa fonction ou sa `const`, et recopie le tri : **dans** la fermeture, la seule occurrence attendue est le `FOR UPDATE`
    de la branche `Lecture::Verrouillante` de `lettering_guard` (`journal_entries.rs:1182`), non prise (`Lecture::Conseil`,
    l. 1261) ; toute autre occurrence dans la fermeture est un défaut à porter au Project Lead. L'exécution le confirme
    pour les écritures (AC11 (a)), et pour les verrous selon la mesure de l'AC11 (c) ; (2) `api_keys::touch_last_used`,
    écrite par le **middleware** pour une requête par clé d'API — hors service, inchangée (point 2 de l'inventaire de la
    32-3).
14. **AC14 — mutations qui doivent rougir** (une à la fois, restauration `git checkout -- <f> && touch <f>`, diff
    résiduel nul ; gate ciblé de l'AC15 ; chacune rougit **au moins une assertion nommée**, recopiée au Dev Agent
    Record avec son diff) — **35** (33 à la fiche, plus M16′ et M17′ à la validation P1) : **34** qui doivent rougir,
    et M8, équivalente déclarée (validation P2, F7) :
    - **M1a-e** — `acces.exiger(…)?` retiré de `ecritures::lister`, `ecritures::detail`, `postes_ouverts::lister`,
      `carnet::lister`, `carnet::detail` → AC9 (a) du service ;
    - **M2a** — `exiger` déplacé **après** les contrôles dans `ecritures::lister` ; **M2b** — après `parse_as_of` dans
      `postes_ouverts::lister` → AC9 (c-val) ;
    - **M3a-d** — `exiger` déplacé **après la première lecture** dans `ecritures::detail`, `postes_ouverts::lister`,
      `carnet::lister`, `carnet::detail` → AC9 (c-lec) ;
    - **M4a** — `EXIGENCE` du carnet à `BooksRead` → AC9 (a) (`{BooksRead, InvoicesRead}` seules) ; **M4b** — celle de
      `ecritures::lister` à `ContactsRead` → AC9 (a) ;
    - **M5** — `role_api` de `carnet::detail` à `Comptable` → AC9 (e) ;
    - **M6a** — plafond du carnet perdu (`clamp(1, MAX)` → `max(1)`) ; **M6b** — plancher perdu (`→ min(MAX)`) ;
      **M6c** — `offset.max(0)` retiré ; **M6d** — défaut 20 remplacé par le plafond → AC10 (b) et (c) ;
    - **M7a-c** — plafond, plancher, `offset` des postes ouverts → AC10 (b) (et `open_items_e2e`) ;
    - **M8** — écrêtage retiré de `ecritures::lister` : **équivalente**, déclarée (le dépôt écrête, `kesh-db/src/repositories/journal_entries.rs:935-936`,
      C-32-4a-11) ; **M8′** — défaut des écritures remplacé par le plafond → AC10 (b) (`None` → 50) ;
    - **M9** — contrôle `accountId <= 0` déplacé avant l'analyse des dates → AC10 (a), cas d'ordre ;
    - **M10** — contrôle « `amountMin` négatif » retiré → AC10 (a) ;
    - **M11** — **filtre de société perdu** : `carnet::detail` appelle `contacts::find_by_id` (sans société,
      `kesh-db/src/repositories/contacts.rs:337`) → AC11 (b) ;
    - **M12** — `carnet::detail` lit la société avant le contact → AC9, cas d'ordre ;
    - **M13** — un lecteur de `ecritures::detail` sur une connexion acquise de la `Lecture` au lieu de `&mut tx` →
      garde déplacée (AC4) ;
    - **M14** — une écriture sur la `Lecture`, **erreur propagée** : `sqlx::query("INSERT INTO audit_log …")
      .execute(lecture.pool()).await.map_err(kesh_db::errors::map_db_error)?;` dans `carnet::detail` (un `let _ =` avalerait la 1792 et
      la mutation survivrait sans que la garde ait faibli ; validation P2, F5) → AC11 (a) (1792), l'erreur rendue
      recopiée au Dev Agent Record ;
    - **M15** — `get_contact` relit le contact par `state.pool` → AC12 (ii) et (iii) (#738 A5 : la garde (ii) rougit
      enfin sous une mutation) ;
    - **M16** — un handler de lecture neuf sur `state.pool` dans `letterings.rs`, écrit **sous la forme coupée** que
      rustfmt donne aux voisins (`let mut conn = state\n        .pool\n        .acquire()…`, seule occurrence du corps) →
      AC12 (iii) ; **M16′** — le même handler déclaré `pub(crate) async fn` → AC12 (iii), compte au niveau du
      fichier (aucun corps ne l'ouvre ; validation P2, R3) ;
    - **M17** — le service `rapports::journaux` trace un filtre **absent** comme un filtre présent (le premier journal ;
      `journal_filter`, `rapports.rs:602`) → ligne 19 de `TABLE_DES_TRACES`, `journaux_affichage_sans_filtre` (AC17 ;
      aucune ligne ne le voyait avant) ; **M17′** — le bras **affichage** de `tracer` passe `None` à
      `emit_report_audit` au lieu de `journal_filter.as_deref()` (`rapports.rs:388`) → ligne 7, restée filtrée — la
      seule qui exerce « affichage avec filtre », la ligne 8 passant par `emit_report_export_audit` (validation P1, R1) ;
    - **M18** — une mention de `Trace` dans le code d'un service du rollout, **sans toucher sa signature** :
      `let _t: Option<&crate::services::poignees::Trace> = None;` dans le corps de `carnet::detail` → AC12 (viii).
      (Un paramètre `trace: &Trace` ajouté casserait la compilation des appelants, E0061 : la mutation ne s'exécuterait
      pas, aucune assertion nommée ne rougirait ; validation P1, R8.)
15. **AC15 — gates.** `kesh-db` n'est **pas** touché (aucun dépôt, aucune migration, ni `post_restore.rs`). En boucle de
    revue, **gate ciblé** : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, puis
    `cargo nextest run -E 'package(kesh-api) & kind(lib)'` et les binaires `services_rollout`, `services_motif`, les
    seize de l'AC8 et les huit gardes de sources (`binary(…)` par nom, liste recopiée au Dev Agent Record). Au **dernier
    commit de code** : **gate complet** (`scripts/test-fast.sh` sous `flock`, bases `kesh_324a` et `kesh_e2e_324a`
    remises à zéro d'abord, sans toucher au conteneur), les **quatre gates frontend** et l'**E2E Playwright complet**
    (recette de `docs/testing.md`, port attribué par l'orchestrateur, jugé fichier par fichier contre § *Les échecs
    attendus*). Ne déclarer que ce qui a tourné, avec les chiffres.
16. **AC16 — documentation.** Rien de visible ne change (C-32-4a-12) : ni manuel, ni CHANGELOG, ni README, ni
    `docs/api-external.md` — **le contrôle se fait quand même** : l. 213, 216-218, 405, 441 relues contre le code
    déplacé (défauts et bornes des postes ouverts et des propositions identiques), et l. 426-433, la table « Refus,
    **dans l'ordre où ils parlent** » des postes ouverts, contre l'ordre de l'AC5 (validation P2, R6) ; `grep -nE "contacts|journal-entries|open-items" docs/manual/fr/*.tex`
    et les PDF aplatis (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `grep -oiE
    ".{60}(par page|pagination|100 contacts|500 écritures).{60}"`) : aucun texte ne décrit autrement les bornes. **Deux
    textes bougent** : la note de `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (l. 273-276) nomme les cinq lectures migrées
    et dit que la liste des écritures ne lit pas la société (C-32-4a-6) ; le doc-comment de `services/mod.rs` (AC1,
    AC13). Aucun écran ne change : recette visuelle sans objet. Les doc-comments de module de
    `routes/{journal_entries,letterings,contacts}.rs` renvoient en une ligne aux services pour les lectures migrées.
17. **AC17 — deux points de #738 repris, un troisième en partie** (C-32-4a-10) : **A5** (AC12 (ii), témoins et M15) ;
    **P2-2** (une ligne de `TABLE_DES_TRACES` qui trace les journaux **sans** filtre : la dix-neuvième,
    `journaux_affichage_sans_filtre`, M17 — la ligne 7 restant filtrée, M17′) ; **B4**, pour sa seule forme « `.pool`
    coupé sur deux lignes », sur les gardes (ii) et (iii) (détecteur aplati de l'AC12, M16) — ses autres formes
    (alias de `Lecture::new`, `<Lecture>::new(`, déstructuration `AppState { pool, .. }`, `ContexteAcces::mcp(` hors
    tests) et la garde (i) restent. Le corps de la PR les nomme ; #738 reste ouverte (`refs`) pour les autres.

## Tâches

- [x] **T0** — Worktree et branche (posés) ; bases `kesh_324a`, `kesh_e2e_324a` créées, migrées depuis les migrations
  du worktree, seedées ; port attribué par l'orchestrateur ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-32-4a/target` ;
  `node_modules` par `cp -al` (`docs/agents-orchestration.md` § 4). Étape 0 : recontrôler au code les lignes citées des
  faits 1 à 9 (une ligne lue citée par fait au Dev Agent Record).
- [x] **T1 (AC9-AC11 — rouge d'abord)** — `tests/common/lecture_seule.rs`, `tests/services_rollout.rs` : en **deux
  commits** — (i) AC10 (c) **et** la mesure de l'AC11 (c) (ni l'un ni l'autre ne réfère aux services neufs), **joués
  sur le code d'avant T2** : c'est le relevé qui établit que la table des bornes et des refus décrit le code
  d'aujourd'hui, et ce que la `Lecture` de test refuse ; (ii) le reste, dont on constate qu'il ne compile pas avant T2.
- [x] **T2 (AC1-AC4, AC7)** — `Portee::ContactsRead` ; `services/ecritures.rs` (`lister`, `detail`, la garde déplacée) ;
  handlers minces ; gate ciblé.
- [x] **T3 (AC5, AC7)** — `services/postes_ouverts.rs` ; `get_open_items` mince ; plafond commun importé par
  `get_lettering_proposals` ; gate ciblé.
- [x] **T4 (AC6, AC7)** — `services/carnet.rs` ; `list_contacts`, `get_contact` minces ; gate ciblé.
- [x] **T5 (AC8, AC12, AC17)** — amendements de `services_motif.rs` (liste fermée de l'AC8 (1)) ; doc-comment de
  `services/mod.rs` (AC1, AC13) ; textes de l'AC16.
- [x] **T6 (AC14)** — mutations M1a à M18, chacune tuée ou déclarée (M8 équivalente).
- [x] **T7 (AC8, AC13, AC15, AC16)** — contrôles de l'AC8, gate complet et E2E complet au dernier commit de code ; Dev
  Agent Record : chiffres réellement exécutés, listes exactes, inventaire.

## Dev Notes

### Sites

- `crates/kesh-api/src/routes/journal_entries.rs` — constantes l. 41-50 ; DTO l. 291-330 ; `list_journal_entries`
  l. 345-487 ; `get_journal_entry` l. 489-532 ; `mod tests` l. 855, garde l. 1005-1080.
- `crates/kesh-api/src/routes/letterings.rs` — constantes l. 286-300 ; `OpenItemsQuery` l. 302-312 ; `parse_as_of`
  l. 504-527 ; `get_open_items` l. 530-551 ; `get_lettering` (l. 236) et `get_lettering_proposals` (l. 557), qui
  restent. Aucun `#[cfg(test)]`.
- `crates/kesh-api/src/routes/contacts.rs` — constantes l. 42-43 ; `ListContactsQuery` l. 48-70 ;
  `contact_response_with_label` l. 510-522 ; `list_contacts` l. 563-602 ; `get_contact` l. 607-622 ; `mod tests` l. 784.
- `crates/kesh-api/src/services/{mod,acces}.rs` — liste des modules (l. 102-105), inventaire (l. 76-89) ; `Portee`.
- `crates/kesh-api/tests/services_motif.rs` — l. 869, 894-899, 833-855, 2073-2100, 2131-2151, 432-457 (lignes 7 et 8
  de la table), 765-777 (son compte), 1998-2033 (`sans_commentaires`, `corps_des_handlers`).
- `crates/kesh-api/src/helpers.rs:76` — `company_for_ids`, réemployée telle quelle.
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md:273-276`.

### Esquisse (indicative ; noms et règles R1-R8 exigés, détail au développement)

```rust
// services/ecritures.rs
pub const LIMITE_DEFAUT: i64 = 50;  pub const LIMITE_MAX: i64 = 500;
pub const EXIGENCE_LISTER: Exigence = Exigence { role_api: Role::Consultation, portee_mcp: Some(Portee::BooksRead) };
pub const EXIGENCE_DETAIL: Exigence = /* idem */;
pub struct DemandeListeEcritures { pub description: Option<String>, pub account_id: Option<i64>,
    pub amount_min: Option<String>, pub amount_max: Option<String>, pub date_from: Option<String>,
    pub date_to: Option<String>, pub journal: Option<Journal>, pub sort_by: SortBy, pub sort_dir: SortDirection,
    pub limit: Option<i64>, pub offset: Option<i64> }
pub async fn lister(acces: &ContexteAcces, lecture: &Lecture, d: DemandeListeEcritures) -> Result<JournalEntryListResult, AppError>;
pub struct DetailEcriture { /* écriture, contre-passée par, blocage contre-passation, blocage modification, pièce */ }
pub async fn detail(acces: &ContexteAcces, lecture: &Lecture, id: i64) -> Result<DetailEcriture, AppError>;
// services/postes_ouverts.rs
pub struct DemandePostesOuverts { pub account_id: i64, pub as_of: Option<String>, pub limit: Option<i64>, pub offset: Option<i64> }
pub async fn lister(acces: &ContexteAcces, lecture: &Lecture, d: DemandePostesOuverts) -> Result<OpenItems, AppError>;
// services/carnet.rs
pub struct PageContacts { pub page: ContactListResult, pub societe: Company }
pub struct FicheContact { pub contact: Contact, pub societe: Company }
pub async fn lister(acces: &ContexteAcces, lecture: &Lecture, d: DemandeListeContacts) -> Result<PageContacts, AppError>;
pub async fn detail(acces: &ContexteAcces, lecture: &Lecture, id: i64) -> Result<FicheContact, AppError>;
```

### Hors périmètre, et ce qui reste non résolu (inventaire)

- **`get_lettering`, `get_lettering_proposals`** — lectures des fichiers visés, non migrées (C-32-4a-1), figées par
  l'AC12 (iii).
- **`contact_persons::list_persons`** (`GET /contacts/{id}/persons`) — autre module, non migré (C-32-4a-2).
- **Les dix mutants** des trois fichiers — C-32-0-4 (0.17.0).
- **Le contrôle de société de la liste des écritures** — absent aujourd'hui, conservé (C-32-4a-6) ; écrit au document
  des motifs.
- **Les factures** (clientes et fournisseur, balance âgée) — 32-4b.
- **#738, points non repris** (C-32-4a-10) : **B1** (`#[must_use]` sur `Rapport`), **B2** (la fermeture `rendre` reçoit
  `&T`), **B3** (accesseurs en `Option`) — conception du `Rapport`, que cette story ne produit ni ne consomme ; la 32-4b
  n'en produit pas davantage (« balance âgée : appel seulement ») : à reprendre quand un outil MCP consommera un
  `Rapport` (32-9a) ; **B4** (gardes non étanches) — repris pour la seule forme coupée, sur (ii) et (iii) (AC12,
  AC17) ; ses autres formes et la garde (i) restent : aucun faux vert aujourd'hui, et la limite est écrite à l'AC12 ;
  **E3**, **E4** (ordre des rapports de projet, trace en échec) — tests des rapports, hors des modules visés ; **B5**
  (doc-comment déplacé dans `routes/reports.rs`) — un sixième fichier de production pour une ligne de commentaire ;
  **A3**, **A4**, **P2-1** (textes de la fiche 32-3) — une fiche close est un relevé daté, non une page à tenir ; **A6**
  — non vérifiable. #738 reste ouverte pour eux.
- **La lecture seule de production** (Q11) — 32-6, comme pour la 32-3.
- **La plage des dates `dateFrom` / `dateTo` de la liste des écritures** (validation P1, R10) — l'analyse
  (`journal_entries.rs:355-369`, `NaiveDate::from_str` seul) n'exige pas la plage 1000-9999 que `parse_as_of` exige
  pour `asOf` (`letterings.rs:504-527`, « revue P1, B-1 = E-1 » de la 15-1b : chrono lit `+10000-01-01` ou une année
  négative, que sqlx refuse d'encoder ou que MariaDB lirait comme une date invalide). La conséquence — un 500 ou un
  résultat faux plutôt qu'un 400 — **n'est pas établie à l'exécution**. Hors périmètre, **motivé** : la story déplace la
  validation **sans la changer** (« messages identiques au caractère près », AC3 ; C-32-4a-5), et l'AC10 (a) fige les
  neuf refus d'aujourd'hui ; ajouter la plage serait un dixième refus, donc un changement de comportement de l'API,
  à décider et à tester pour lui-même. Le service en devient l'unique porte pour le MCP (C-32-4a-5) : la correction,
  quand elle viendra, se fera **une fois**, au service. **Issue ouverte : #742** (texte au Change Log, validation P1 ;
  reproduction corrigée en `dateFrom=%2B10000-01-01`, validation P2, F3, versée en commentaire sur #742).
- **Deux commentaires de `kesh-db` que la story rend faux** (validation P2, R7) — `kesh-db/src/repositories/journal_entries.rs:775-776`
  (« la source canonique est le route handler ») et `:934` (« Clamp défensif — la source de vérité est le route
  handler ») : après l'AC3, la source est `ecritures::lister`. `kesh-db` n'est pas touché (AC15) ; « on trace
  toujours » : **à tracer par l'orchestrateur** (issue P3, ou point ajouté à #738). Le doc-comment des postes ouverts
  (« l'appelant les écrête », `open_items.rs:337-339`) reste vrai.

### Fichiers touchés (prévus)

`crates/kesh-api/src/services/{ecritures,postes_ouverts,carnet}.rs` (neufs) ; `services/mod.rs`, `services/acces.rs` ;
`crates/kesh-api/src/routes/{journal_entries,letterings,contacts}.rs` ; `crates/kesh-api/tests/services_rollout.rs` et
`tests/common/lecture_seule.rs` (neufs) ; `tests/common/mod.rs`, `tests/services_motif.rs` ;
`docs/MULTI-TENANT-SCOPING-PATTERNS.md`. **Non touchés** : `kesh-db`, `lib.rs` (aucune route ni poignée neuve),
`helpers.rs`, frontend, i18n, manuels, CHANGELOG.

**Découpage préventif** : modules de premier niveau touchés — `kesh-api/services`, `kesh-api/routes/journal_entries`,
`kesh-api/routes/letterings`, `kesh-api/routes/contacts` — **4**, comme l'annonçait l'epic ; tests et un document en
plus. Sous le seuil de 5 : **non découpée**.

### Risques

- **Moyen — une borne perdue.** Contacts et postes ouverts n'ont d'autre borne que celle qu'on déplace (fait 3) ;
  T1 (i) fige les bornes **par la route avant** le déplacement ; M6, M7.
- **Moyen — un message de validation qui change.** Les neuf refus des écritures ne sont figés par aucun test (fait 4) :
  la table de l'AC10 (a) se joue avant T2 par la route (T1 (i) en porte au moins un) puis au service.
- **Faible — la garde de transaction déplacée** perd sa force si ses assertions s'affaiblissent au déplacement : diff
  recopié (AC4), M13.
- **Faible — ordre des lectures** (contact avant société ; `asOf` avant la société) : AC9, M12, M2b.
- **Faible — conflit avec la 32-4b** (validation P1, F8) : le graphe de l'epic met 32-4a et 32-4b côte à côte
  (`epic-32-serveur-mcp.md:213`) ; les deux touchent `tests/services_rollout.rs` (créé ici) et le compte **absolu** des
  fichiers de `services/` (`services_motif.rs:2137-2142`, huit ici). **Ordre voulu** : la 32-4b part de `main` **après**
  la fusion de la 32-4a (C-32-4a-9). À défaut — si l'orchestrateur les mène en parallèle —, celle qui fusionne **en
  second** rebase sur l'autre, ajoute ses tests à `services_rollout.rs` sans le recréer et porte le compte de
  `services/` à la somme.

### Références

`CLAUDE.md` § *Test Locally First*, § *Pendant une boucle de revue*, § *Règle de splitting préventif*, § *Inventorier
les sites NON RÉSOLUS*, § *Recompter ses propres comptes rendus* ; `docs/agents-orchestration.md` § 2-7 ;
`planning-artifacts/epic-32-serveur-mcp.md` § *32-4*, § *32-9* ; fiche `32-3-services-motif.md` (§ *Le motif*, AC7,
AC8, AC10, R1-R8) ; registres `epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7) et `-323.md` (C-32-3-4, C-32-3-36 à
41) ; #738.

## Points à soumettre à Guy

- **Q1 — l'écart de C-32-4a-1 à la lettre de l'AC de l'epic** (validation P1, R7). L'AC de la 32-4 écrit « aucun
  handler de lecture des modules visés n'appelle plus un dépôt directement » (`epic-32-serveur-mcp.md:315-316`) ; la
  story en migre **cinq** sur **sept** et laisse `get_lettering` (`GET /letterings/{key}`) et `get_lettering_proposals`
  (`GET /accounts/{id}/lettering-proposals`) sur `state.pool`, parce qu'aucun outil prévu de la 0.16.0 ne les sert
  (C-32-0-4) — nommés à l'inventaire des non résolus et figés par la garde (AC12 (iii)). Le registre l'a tranché seul ;
  l'epic est annoté d'une ligne datée qui renvoie au registre. **La question** : acceptez-vous que l'AC de l'epic se lise
  « aucun handler de lecture **servi par un outil** », les deux autres restant inventoriés — ou faut-il migrer les sept
  (deux services sans consommateur, une story un peu plus large, toujours sous le seuil de découpage) ? Le
  développement peut partir sur la première lecture ; la seconde se rattrape sans défaire le travail (deux services de
  plus, l'inventaire de l'AC12 (iii) passant de douze à dix).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (claude-opus-5-5), en autonomie, worktree `kesh-32-4a`, base de développement `39939799` (fiche validée).
Port 3326, bases `kesh_324a` (gate) et `kesh_e2e_324a`, cible `kesh-32-4a/target`. **Q1** non tranchée : développé sur
la première lecture (cinq lectures migrées, les deux de lettrage inventoriées et figées).

### Completion Notes List

#### Étape 0 — prémisses recontrôlées au code (ligne lue citée)

- **P3-1** — confirmé : la requête de `find_source_document_of_entry` est aux l. **387-396** de
  `kesh-db/src/repositories/imported_supplier_invoices.rs` (l. 387 `sqlx::query_as::<_, EntrySourceDocument>(`,
  l. 396 `ORDER BY si.id LIMIT 1",`).
- **P3-2** — confirmé : `services_motif.rs:2091` — `.find(|&j| lignes[j].contains("pub async fn"))`.
- **P3-3** — confirmé, et appliqué en **rétablissant** le compte des `async fn` toutes visibilités à côté du compte au
  niveau du fichier (C-32-4a-13) ; relevé `awk … | grep -cE '^\s*(pub(\([^)]*\))?\s+)?async fn'` → 7 / 5 / 5.
- **P3-4** — appliqué : les limites de la garde (doc-comment de `ac12_iii_inventaire_des_sites_non_resolus`) nomment
  `self.pool` (`impl AppState`), `(&state).pool`, `state.clone().pool`, une macro à paramètre ident, la
  déstructuration et la liaison renommée.
- **P3-5** — appliqué par le code : les autres formes de #738 B4 sont écrites aux limites, dans la garde ; la fiche
  n'est pas réécrite (relevé daté).
- **P3-6** — appliqué : M14 joue `INSERT INTO audit_log (user_id, action, entity_type, entity_id) VALUES (1,
  'contact.read', 'contact', 0)`, instruction complète (patron du témoin `ac9_temoin_la_lecture_de_test_refuse_l_ecriture`).
- **P3-7** — appliqué : `BORNE_*` et `EXERCICE_*` non recopiées dans `services_rollout.rs` (clippy `-D warnings` vert).
- **P3-8** — appliqué : C-32-4a-14 au registre ; le commentaire de #738 sur `journal_entries.rs:775-776` et `:934` de
  `kesh-db` vérifié (`gh issue view 738 --json comments` : deuxième commentaire, « relevé par la validation P2 de la
  fiche de la Story 32-4a »).

Faits 1 à 9 (une ligne lue par fait, au code de `39939799`) : (1) `grep -n '^pub async fn'` → 7 / 5 / 5, dont
`letterings.rs:534:pub async fn get_open_items(` ; (2) `lib.rs` : `.route("/api/v1/contacts", get(routes::contacts::list_contacts))`
dans `authenticated_routes` ; (3) `kesh-db/src/repositories/journal_entries.rs:935` `let clamped_limit =
query.limit.clamp(1, MAX_LIMIT);`, `contacts.rs:472` `items_qb.push_bind(query.limit);` ; (4)
`routes/journal_entries.rs:359` `.map_err(|e| AppError::Validation(format!("dateFrom invalide ({e})")))?,` ; (5)
`:506` `let mut tx = state.pool.begin().await.map_err(map_db_error)?;` ; (6) `kesh-db/…/journal_entries.rs:1182`
`Lecture::Verrouillante => format!("{LETTERING_GUARD_SQL} FOR UPDATE"),`, `:1261` `lettering_guard(&mut *conn, id,
Lecture::Conseil)` ; (7) `routes/contacts.rs:613` `find_by_id_in_company(&state.pool, …)` avant `:618`
`get_company_for` ; (8) `services/mod.rs:100` `#![cfg_attr(not(test), forbid(unsafe_code))]` ; (9)
`docs/MULTI-TENANT-SCOPING-PATTERNS.md:273` `- **Every handler validates company exists**`.

#### T1 — rouge d'abord

- **T1 (i)** (`c550ea8c`), joué **sur le code d'avant T2** : AC10 (c) par la route (contacts `limit=1000&offset=-3` →
  100 / 0, défaut 20 ; écritures `limit=0` → 1, défaut 50 ; `dateFrom=x` → 400 `VALIDATION_ERROR`, message
  `dateFrom invalide (…`) et la **mesure de l'AC11 (c)** : `SELECT 1 FROM fiscal_years LIMIT 1 FOR UPDATE` sur
  `pool_lecture_seule` (session `SET SESSION TRANSACTION READ ONLY`) → **refusé, erreur 1792, en autocommit ET dans
  `begin()`** (MariaDB 10.11.19 du conteneur de dev, `SELECT VERSION()`) ; figé par `ac11_c_un_verrou_est_refuse_sur_la_lecture_de_test`. Les
  deux tests verts sur le code d'avant : ils décrivent le code d'aujourd'hui. Conséquence (AC11 (c)) : l'AC11 (a)
  confirme aussi l'absence de verrou sur les branches exercées ; le doc-comment de `services/mod.rs` le dit.
- **T1 (ii)** (`7abc9073`) : constaté **ne compile pas** avant T2, sources neuves mises de côté (`git stash -u` des
  seules sources de `src/`) — E0432 sur `services::{ecritures, postes_ouverts, carnet}`, E0599 ×3 sur
  `Portee::ContactsRead` (journal `324a-t1ii-avant-t2.log`).

#### T2 à T5 — le code (`90dbcf5b`, puis `345bcc7f` pour les doc-comments)

- **Un commit pour T2 à T4** (C-32-4a-17) : les tests de T1 (ii) importent les trois services.
- **Conversion `CoreJournal → DbJournal` au service** (C-32-4a-15). Les DTO gardent leur forme et leurs défauts serde,
  qui rendent les constantes du service ; les handlers passent `Some(params.limit)` (C-32-4a-16).
- **Garde déplacée** (AC4, C-32-4a-7), nom conservé : `services::ecritures::tests::get_journal_entry_reads_in_one_transaction`.
  Diff jeton par jeton contre l'ancienne (`routes/journal_entries.rs:1005-1080` à `39939799`), aucun motif relâché :
  - `include_str!("journal_entries.rs")` → `include_str!("ecritures.rs")` ;
    `find("pub async fn get_journal_entry(")` / `"get_journal_entry introuvable"` → `find("pub async fn detail(")` /
    `"detail introuvable"` ;
  - `corps.matches("pool.begin()").count()` → `sans_blancs.matches("pool().begin()").count()`, `sans_blancs` = le corps
    débarrassé de ses blancs (`corps.chars().filter(|c| !c.is_whitespace())`) ;
  - `!appels[0].contains("state.pool")` → `!appels[0].contains(".pool()")` ;
  - `corps.contains("journal_entries::find_by_id(&state.pool")` → `corps.contains("journal_entries::find_by_id(lecture.pool()")` ;
  - inchangés : `take_while(… != "#[cfg(test)]")`, filtre `//`, fin au `\npub async fn ` suivant, `acquire()` compté
    zéro, les quatre couples (module, lecteur), « un appel » et les formes `(&mut tx,` / `(&mut *tx,`.
- **AC7, contrôles au code** :
  `grep -nE "\b(MAX_LIMIT|DEFAULT_LIMIT|MAX_LIST_LIMIT|DEFAULT_LIST_LIMIT|OPEN_ITEMS_DEFAULT_LIMIT)\b" crates/kesh-api/src/routes/{journal_entries,letterings,contacts}.rs`
  → **rien** ; `grep -nE 'clamp\(|\.max\(0\)' …` → **deux** lignes, dans `get_lettering_proposals` :
  `letterings.rs:544:    // \`clamp(1, 500)\` : la valeur est positive, la conversion ne perd rien.` et
  `letterings.rs:545:    let limit = params.limit.clamp(1, postes_ouverts::LIMITE_MAX) as usize;`.
- **AC8** : `git diff --diff-filter=MDR --name-only $(git merge-base HEAD origin/main) HEAD -- crates/kesh-api/tests`
  → `crates/kesh-api/tests/common/mod.rs`, `crates/kesh-api/tests/services_motif.rs` ; `diff` du `mod tests` de
  `contacts.rs` → vide ; celui de `journal_entries.rs` → la seule suppression `150,226d149` (le bloc de la garde,
  aucune ligne ajoutée). Modifications de `services_motif.rs`, liste fermée : `mod common;` + import de l'aide
  commune, aides `pool_lecture_seule`/`lecture_seule` retirées, `TOUTES` à trois portées, bras `ContactsRead` à
  `autre_portee` (`panic!`, C-32-4a-18), ligne 19 `journaux_affichage_sans_filtre` et le compte (« dix-neuf lignes pour
  dix-huit couples ») avec le doc-comment de `table_des_traces_par_la_route`, `services/` à huit fichiers, la garde de
  l'AC12 ((ii) réécrite, (iii) et (viii) neuves, témoins).
- **AC9, cas sans objet** (dits aussi au doc-comment de `ac9_c_le_refus_precede_validation_et_lecture`) : (c-lec)
  pour `ecritures::lister` ; (c-val) pour `ecritures::detail`, `carnet::lister`, `carnet::detail`.
- **AC11 (a), branches non atteintes** par les fixtures (non revendiquées) : pièce source présente, motifs de blocage
  de la contre-passation et de la modification présents, compte non lettrable, contact avec conditions de paiement.
- **AC12 (iii)**, relevé à `345bcc7f` : douze couples (fichier, handler) sur `state.pool`, 7 / 5 / 5 `pub async fn`
  et `async fn`, compte du fichier égal à la somme des corps (test vert).
- **AC13 — tri des écritures sur la fermeture** (`324a-ac13-tri.txt`) : `grep -nE "INSERT|UPDATE |DELETE|FOR
  UPDATE|get_or_create"` sur le code (lignes de commentaire écartées) des huit fichiers, chacun tronqué à son
  `#[cfg(test)]` (`journal_entries.rs:3111`, `fiscal_years.rs:1394`, `letterings.rs:2215`, `open_items.rs:618`,
  `contacts.rs:738`, `search.rs:158` ; aucun dans `imported_supplier_invoices.rs` ni `companies.rs`) → **77**
  occurrences : `journal_entries.rs` 20, `fiscal_years.rs` 15, `imported_supplier_invoices.rs` 8, `letterings.rs` 10,
  `open_items.rs` 0, `contacts.rs` 5, `companies.rs` 19, `search.rs` 0, rangées sous 49 fonctions ou `const`. **Dans la
  fermeture, une seule** : `journal_entries.rs:1182`, `fn lettering_guard`, branche `Lecture::Verrouillante`, non
  prise. Toutes les autres sont dans des fonctions ou `const` hors fermeture (`create_in_tx_inner`, `update_in_tx`,
  `delete_in_tx`, `reverse_in_tx_inner`, `create_opening_entry`, `delete_all_by_company`, `find_later_closed_in_tx`,
  `find_open_covering_date`, `find_overlapping`, `find_by_name`, `LOCK_*_SQL`, `SYNC_*_SQL`, `create`/`update`/`archive`
  des contacts, mutations de `companies`…). Aucun défaut à porter au Project Lead. Point (2) : `api_keys::touch_last_used`,
  middleware, inchangé.
- **AC16** : `api-external.md` l. 213, 216-218 (routes), 405 (`limit` défaut 50, 1-500 ; `offset` défaut 0, ramené à 0),
  426-433 (refus dans l'ordre `asOf` 400 → 404 → 409) et 441 (propositions défaut 100, plafond 500) relues contre le
  code déplacé : conformes, aucune modification. `grep -nE "contacts|journal-entries|open-items" docs/manual/fr/*.tex`
  → mentions générales (admin l. 1041, 1414, 1464, 2195, 2220, 2328, 2471, 2902 ; user l. 913, 933), aucune borne ;
  PDF aplatis (`admin-manual`, `user-manual`, `marketing-brochure`) : `grep -oiE ".{60}(par page|pagination|100
  contacts|500 écritures).{60}"` → rien. Textes modifiés : la note de `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, le
  doc-comment de `services/mod.rs`, les doc-comments de module des trois fichiers de routes.

#### T6 — mutations (AC14), pilote `kesh-gate-logs/324a-mutations-pilote.py`, journal `324a-mutations-campagne.log`

Une à la fois, sous verrou, motif vérifié unique, restauration `git checkout -- <f> && touch <f>`, diff résiduel nul
après chacune ; journal et diff par mutation : `324a-mut-<id>.{log,diff}` (`′` écrit `p`). **34 tuées, M8 survivante
(équivalente déclarée)** :

| Mutation | Assertion(s) nommée(s) qui rougissent |
|---|---|
| M1a-e (`exiger` retiré, cinq services) | `ac9_a_b_e_refus_et_controle_positif`, `ac9_c_le_refus_precede_validation_et_lecture` |
| M2a, M2b (`exiger` après la validation) | `ac9_c_…` |
| M3a-d (`exiger` après la première lecture) | `ac9_c_…` |
| M4a (carnet `BooksRead`) | `ac9_a_b_e_…`, `ac9_c_…`, `ac9_carnet_detail_lit_le_contact_avant_la_societe` |
| M4b (écritures `ContactsRead`) | `ac9_a_b_e_…`, `ac9_c_…` |
| M5 (`carnet::detail` `Comptable`) | `ac9_a_b_e_…`, `ac11_a_les_cinq_services_lisent_en_lecture_seule` |
| M6a, M6c, M6d (carnet : plafond, `offset`, défaut) | `ac10_b_les_bornes_des_trois_listes`, `ac10_c_bornes_et_refus_par_la_route` |
| M6b (carnet : plancher) | `ac10_b_…` |
| M7a-c (postes ouverts) | `ac10_b_…`, `open_items_e2e::inputs_defaults_bounds_and_refusal_order` |
| **M8** (écrêtage des écritures retiré) | **survit — équivalente** : le dépôt écrête (`kesh-db/src/repositories/journal_entries.rs:935-936`, C-32-4a-11) |
| M8′ (défaut des écritures au plafond) | `ac10_b_…` |
| M9 (`accountId` avant les dates) | `ac10_a_les_neuf_refus_de_la_liste_des_ecritures` |
| M10 (`amountMin` négatif retiré) | `ac10_a_…` |
| M11 (`contacts::find_by_id`, sans société) | `ac11_b_aucune_lecture_d_une_autre_societe` |
| M12 (société avant le contact) | `ac9_carnet_detail_lit_le_contact_avant_la_societe` |
| M13 (lecteur sur une connexion acquise) | `services::ecritures::tests::get_journal_entry_reads_in_one_transaction` |
| M14 (`INSERT` propagé dans `carnet::detail`) | `ac11_a_…` — erreur rendue : `AC11 (a) carnet::detail : Err(Database(Sqlx(Database(MySqlDatabaseError { code: Some("25006"), number: 1792, message: "Cannot execute statement in a READ ONLY transaction" }))))` |
| M15 (`get_contact` relit par `state.pool`) | `ac10_i_ii_handlers_sans_le_pool`, `ac12_iii_inventaire_des_sites_non_resolus` |
| M16, M16′ (handler neuf, forme coupée ; `pub(crate)`) | `ac12_iii_…` |
| M17, M17′ (journaux : filtre absent tracé présent ; affichage à `None`) | `ac9_b_table_des_traces_au_service`, `table_des_traces_par_la_route` |
| M18 (`Trace` nommée dans `carnet::detail`) | `ac12_viii_les_services_du_rollout_ne_tracent_pas` |

Incident de pilote, sans effet : le premier passage s'est arrêté à M15 sur un motif **non unique** (trois sites dans
`routes/contacts.rs`), **avant** toute écriture ; motif resserré au corps de `get_contact`, M15 à M18 rejouées en
second passage (même journal).

#### T7 — gates (au dernier commit de code, `345bcc7f`)

- **Gate ciblé de l'AC15** (`324a-cible-ac15.log`, filtre `324a-filtre-cible.txt`) : **984 tests, 984 passés, 1
  ignoré**, 27 binaires — `kesh-api` lib 529, `services_rollout` 9, `services_motif` 23 ; les seize de l'AC8 :
  `admin_full_import_e2e` 54, `contact_client_number_e2e` 8, `contact_payment_terms_e2e` 9, `contact_persons_e2e` 4,
  `filet_bilan_clos_e2e` 2, `idor_multi_tenant_e2e` 14, `inbox_import_e2e` 39, `invoice_ttc_e2e` 3,
  `journal_entry_reversal_e2e` 55, `kf004_no_op_e2e` 6, `letterings_e2e` 11, `onboarding_e2e` 16,
  `opening_balances_e2e` 31, `open_items_e2e` 7, `period_lock_e2e` 16, `reports_e2e` 38 ; les huit gardes :
  `audit_label_registry` 11, `textes_coherents` 19, `admin_pat_denied_e2e` 16, `audit_route_registry` 11,
  `test_schema_guard` 7, `configuration_transmise` 30, `bank_account_closed_inventory` 9, `letterings_lexical` 7.
  (Les comptes `grep -c 'sqlx::test'` de la fiche — 56, 40, 56, 39 — comptent aussi des mentions hors attribut ; les
  nombres ci-dessus sont ceux **exécutés**.) Frappent l'une des cinq routes en `GET` (relevé au site) :
  `contact_client_number_e2e`, `contact_payment_terms_e2e`, `idor_multi_tenant_e2e`, `inbox_import_e2e`
  (`GET /journal-entries/{id}`), `journal_entry_reversal_e2e`, `letterings_e2e`, `open_items_e2e`.
- **Gate complet** (`324a-gate-complet.log`, rév. `345bcc7f`, bases `kesh_324a` et `kesh_e2e_324a` remises à zéro
  avant, conteneur non touché) : `scripts/test-fast.sh --ci` sous verrou — fmt, clippy `-D warnings`, garde des droits
  conforme, **nextest 4013 exécutés, 4013 passés, 7 ignorés**.
- **Frontend** (`324a-frontend.log`, rév. `345bcc7f`) : `npm run check` 0 erreur (29 avertissements ; la story ne touche aucun fichier de `frontend/` — `git diff 39939799...HEAD --stat -- frontend` vide) ;
  `lint-i18n-ownership` PASS ; `test:unit` **162 fichiers, 1911 tests** passés ; `build` vert.
- **E2E complet** (`324a-e2e.log`, rév. `345bcc7f`, script `324a-e2e.sh` — recette de `docs/testing.md`, port 3326,
  base `kesh_e2e_324a`, dossiers `/tmp/kesh-e2e-324a/` passés au backend et au runner ; backend arrêté par PID, port
  libre ensuite) : **315 passés, 7 échecs, 17 ignorés** (12,9 min). Jugés fichier par fichier contre `docs/testing.md`
  § *Les échecs attendus* : les sept sont les sept lignes KF-029 (#97) du tableau — `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` ; ni KF-052, ni pollution. Compte 7,
  borne basse de la baseline (7 à 10). Aucune régression.

#### Écarts

- **Édition hors de l'outil Edit** (consigne 14) : la seconde moitié des tests de `services_rollout.rs` (fichier neuf
  de la story, avant son premier commit de T1 (ii)) a été **ajoutée par `cat >> … <<'EOF'`** — un ajout, non une
  substitution, mais pas par l'outil Edit. Aucune autre écriture de code hors Edit/Write ; `cargo fmt` a reformaté.
- T2 à T4 en un commit (C-32-4a-17).

### File List

- `crates/kesh-api/src/services/ecritures.rs` (neuf)
- `crates/kesh-api/src/services/postes_ouverts.rs` (neuf)
- `crates/kesh-api/src/services/carnet.rs` (neuf)
- `crates/kesh-api/src/services/mod.rs`
- `crates/kesh-api/src/services/acces.rs`
- `crates/kesh-api/src/routes/journal_entries.rs`
- `crates/kesh-api/src/routes/letterings.rs`
- `crates/kesh-api/src/routes/contacts.rs`
- `crates/kesh-api/tests/services_rollout.rs` (neuf)
- `crates/kesh-api/tests/common/lecture_seule.rs` (neuf)
- `crates/kesh-api/tests/common/mod.rs`
- `crates/kesh-api/tests/services_motif.rs`
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md`
- `_bmad-output/implementation-artifacts/324a-review-prompt-p1.md` (neuf)
- `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-324a.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/32-4a-services-ecritures-contacts.md`

## Change Log

- **2026-10-24 — fiche rédigée** (Opus 5.5, en autonomie, worktree `kesh-32-4a`, base `89d68704`) : 17 AC, 33
  mutations nommées (M8 déclarée équivalente d'avance), 4 modules (non découpée) ; registre
  `epic-32-choix-autonomes-324a.md`, C-32-4a-1 à 12 ; deux points de #738 repris (A5, P2-2). Validation P1 à lancer
  (`324a-validate-prompt-p1.md`, Sonnet, lentilles R et F).
- **2026-10-24 — validation P1** (Sonnet ×2, contexte frais, lecture seule ; rapports hors dépôt
  `kesh-gate-logs/324a-validate-p1-{R,F}.md`) : **R 0/0/2/8, F 0/0/1/7**, soit 18 findings, **14 distincts** après
  recoupement (R2 = F2, R3 = F3, R4 = F5, R5 = F4) — **3 MEDIUM** (R1, R2 = F2, F1), **11 LOW** ; 0 au-dessus de
  MEDIUM. Remédiation par l'orchestrateur de la fiche (Opus 5.5), **chaque prémisse relue au code** avant application,
  ligne citée :
  - **R1 (MEDIUM)** — appliqué, en ajoutant plutôt qu'en déplaçant : la ligne 7 de `TABLE_DES_TRACES`
    (`services_motif.rs:432-442`) est **la seule** qui exerce « affichage avec filtre » (le bras `tracer` d'affichage,
    `rapports.rs:378-390`, appelle `emit_report_audit` ; la ligne 8 passe par `emit_report_export_audit`, l. 397-409) ;
    la passer à `None` aurait rendu M17′ survivante. Retenu : **une dix-neuvième ligne** sans filtre, la 7 restant
    filtrée ; M17′ ajoutée ; compte de la table à 19 (AC8 (1), AC14, AC17, C-32-4a-10).
  - **R2 = F2 (MEDIUM / LOW)** — appliqué : `letterings.rs:243-244`, `:544-545`, `:566-567` portent bien `state` / `.pool`
    sur deux lignes ; détecteur de (ii)/(iii) sur le corps aplati (`\bstate\s*\.\s*pool\b` après `sans_commentaires`),
    témoin coupé, M16 en forme coupée ; et la remarque de R2 sur les visibilités (`corps_des_handlers` n'ouvre que sur
    `pub async fn`) : compte des `async fn` toutes visibilités (7 / 5 / 5 aujourd'hui, vérifié) et M16′. #738 B4 repris
    pour cette seule forme (AC12, AC17, *Hors périmètre*).
  - **F1 (MEDIUM)** — appliqué : fermeture des appels suivie jusqu'aux requêtes (fait 6, appels directs et transitifs,
    `const` SQL) ; un seul `FOR UPDATE` dans la fermeture, branche non prise (`journal_entries.rs:1182`, `Lecture::Conseil`
    l. 1261) ; AC13 sur les fichiers, commentaires écartés. Sur le `FOR UPDATE` en lecture seule : le dépôt l'a
    **mesuré** refusé (1792) pour `START TRANSACTION … READ ONLY` (`kesh-db/tests/closing_checks.rs:253-274`, 29-1),
    **non** pour `SET SESSION TRANSACTION READ ONLY` — « à confirmer à l'exécution par l'AC11 » était trop fort ; écrit
    « non établi », et mesuré par une AC11 (c) neuve, au patron de la T0 de la 29-1.
  - **R3 = F3** — appliqué : trois jetons, non deux (`pool.begin()`, l. 1030, compterait zéro sur `lecture.pool().begin()`) ;
    compte sur le corps sans blancs (AC4, C-32-4a-7). **R4 = F5** — appliqué : `AppError::Database(DbError::NotFound)`,
    `matches!` (`errors.rs:138-170`) (AC4, AC6, AC9, AC11). **R5 = F4** — appliqué : `mod common;` à l'AC8 (1) ; et, de F4,
    la recopie du montage HTTP et des constructeurs de contexte **assumée** (convention : `spawn_app` dans 68 binaires,
    `forge_jwt` dans 29). **R6** — appliqué : seconde société par `companies::create` (`uq_users_username`). **R8** —
    appliqué : M18 sans toucher la signature. **R9** — appliqué avec F1. **F6** — appliqué : réversibilité de C-32-4a-1
    sans échéance (les « propositions » de la 0.17.0 sont celles de l'agent, `epic-32-serveur-mcp.md:69`). **F7** —
    appliqué : second contrôle sur les écrêtages (`grep -nE 'clamp\(|\.max\(0\)'`, huit lignes aujourd'hui, deux
    attendues). **F8** — appliqué : ordre avec la 32-4b (*Risques*, C-32-4a-9).
  - **R7** — annoté, non tranché : ligne datée à l'AC de la 32-4 dans `epic-32-serveur-mcp.md`, question **Q1** posée
    (*Points à soumettre à Guy*).
  - **R10** — laissé **hors périmètre**, motivé (comportement inchangé ; un dixième refus se décide pour lui-même) ;
    **issue à ouvrir** par l'orchestrateur. Texte prêt : titre « `GET /journal-entries` : `dateFrom` / `dateTo` hors de
    la plage 1000-9999 ne sont pas refusés en 400 » ; labels `bug`, `triage`, `P2` (à confirmer : P2 si l'exécution
    rend un 500) ; corps — « `list_journal_entries` (`crates/kesh-api/src/routes/journal_entries.rs:355-369`) analyse
    `dateFrom` et `dateTo` par `NaiveDate::from_str` seul. `parse_as_of` (`routes/letterings.rs:504-527`) ajoute la
    plage 1000-9999 pour `asOf`, parce que chrono lit `+10000-01-01` ou une année négative que sqlx refuse d'encoder ou
    que MariaDB lirait comme une date invalide (revue P1, B-1 = E-1, Story 15-1b). La liste des écritures n'a pas cette
    garde : `GET /api/v1/journal-entries?dateFrom=+10000-01-01` rend vraisemblablement un 500, ou un filtre faux, au lieu
    d'un 400 `VALIDATION_ERROR` — non vérifié à l'exécution. Avec la Story 32-4a, l'analyse passe au service
    `ecritures::lister`, que l'outil MCP de la 32-9a appellera : la correction se fera au service, une fois. Hors du
    périmètre de la 32-4a, qui déplace la validation sans la changer. À reproduire, puis : refus 400 aux messages
    `dateFrom invalide (année hors de 1000 à 9999)` / `dateTo …`, au patron de `asOf`, test au service et par la route. »
    *(Annoté le 2026-10-24, validation P2, F3 : écrite telle quelle, la reproduction transmet `" 10000-01-01"` — `+`
    se lit comme une espace dans une chaîne de requête — et rendrait un 400 ; elle s'écrit `dateFrom=%2B10000-01-01`
    (ou `dateFrom=-0001-01-01`), au patron de `open_items_e2e.rs:309`. L'issue est ouverte, **#742**, la forme corrigée
    versée en commentaire.)*
  - **Propagation** (grep des valeurs sur la fiche, le registre et `sprint-status.yaml`) : `33` → `35` dans les entrées
    **vivantes** (AC14, ligne de `sprint-status.yaml` par annotation datée), non dans l'entrée « fiche rédigée »
    ci-dessus, relevé daté ; `deux jetons`, `journal: None`, `835-851` / `835-853` (→ `833-855`), `503-527` (→ `504-527`),
    `` `NotFound` `` nu, `0.17.0 (propositions)` traités au registre et à la fiche ; `dix-huit` : seul le compte de la
    table change, le doc-comment du module (« dix-huit handlers ») reste vrai.
  - **Trend** : P1 = 3 MEDIUM (+ 11 LOW). Les trois MEDIUM touchent des **règles de garde** (table des traces, garde
    lexicale, inventaire des écritures) : la P2 est une passe **complète**, **Opus**, lentilles R et F sur la fiche
    entière (`324a-validate-prompt-p2.md`).
- **2026-10-24 — validation P2** (Opus ×2, contexte frais, lecture seule, passe complète ; rapports hors dépôt
  `kesh-gate-logs/324a-validate-p2-{R,F}.md`) : **R 0/0/1/7, F 0/0/0/7**, soit 15 findings, **13 distincts** après
  recoupement (R1 = F1, R2 = F2) — **1 MEDIUM** (R3), **12 LOW**. Trend : P1 3 MEDIUM → P2 1 MEDIUM, et ce MEDIUM
  **naît de la remédiation P1** (le compte des `async fn` et la phrase « la limite qui reste » de l'AC12) : la sévérité
  s'est déplacée vers ce qu'on venait d'écrire. Remédiation par l'orchestrateur de la fiche (Opus 5.5), chaque prémisse
  relue au code :
  - **R3 (MEDIUM)** — appliqué, sous la forme la plus simple. **Une** définition du corps, nommée à l'AC12 : celle de
    `corps_des_handlers` (`services_motif.rs:2009-2011` : « de la ligne d'ouverture à la première ligne égale à `}` ») ;
    la (ii) de `list_accounts` s'y ramène (`accounts.rs` ne porte entre les deux que des doc-comments). L'aide
    synchrone hors corps : contrôlée **au niveau du fichier** — sur les trois fichiers, occurrences du détecteur sur le
    fichier = somme sur les corps ; relevé au code par `awk` (corps jusqu'à la première `}`) puis `tr '\n' ' ' | grep -oE
    '\bstate\s*\.\s*pool\b'` : **15 / 10 / 12** des deux côtés, aucune occurrence hors d'un corps, aucun `/*` — pas de
    faux rouge aujourd'hui. Ce compte **remplace** celui des `async fn` toutes visibilités (P1), qu'il couvre (une
    fonction non `pub` n'ouvre aucun corps) ; M16′ le vise. Non posé sur `accounts.rs`, qui porte `state.pool` hors
    corps (`response`, l. 185-190) : limite écrite. La phrase « la limite qui reste : une déstructuration » devient
    **les limites qui restent** — champ sous un autre nom que `state` (déstructuration, liaison renommée), dans un
    handler comme dans une aide, et une aide appelée par `list_accounts`. Aucune mutation neuve (35 inchangé).
  - **R1 = F1** — appliqué : `imported_supplier_invoices.rs:373-378` (doc-comment) et l. 386-395 (requête sans verrou),
    au lieu de la l. 171 (`find_by_supplier_invoice_id_scoped`).
  - **R2 = F2** — appliqué : troisième puce du fait 6 (aides pures, `const` et textes SQL, premier `SELECT` de
    `modification_blocker` l. 1222-1229), et la fermeture définie comme « toute fonction ou `const` atteinte depuis un
    appel direct » ; AC13 : `util/search.rs` ajouté, chaque fichier tronqué à son `#[cfg(test)]` (lignes relues).
  - **R4** — appliqué : esquisse en `acces`, `lecture` (`plan_comptable.rs:35-36`), et l'AC4 dit que le jeton (2) en
    dépend. **R5** — appliqué : `open_items_e2e.rs:529-539` fige les propositions (fait 3, AC10 (c)). **R6** — appliqué :
    `api-external.md:426-433` à l'AC16 et à C-32-4a-12. **R7** — appliqué : les deux commentaires de `kesh-db`
    (`journal_entries.rs:775-776`, `:934`) nommés au *Hors périmètre*, **à tracer par l'orchestrateur** ; `kesh-db` non
    touché. **R8** — appliqué : défauts du tri des contacts à l'AC6 (`contacts.rs:584-585`).
  - **F3** — appliqué : annotation datée sous le texte d'issue de la P1 (`%2B10000-01-01`) ; #742 ouverte, le
    commentaire déjà versé ; *Hors périmètre* et en-tête renvoient à #742. **F4** — appliqué : `services_motif.rs:43-141`
    (AC9, AC10 (c)). **F5** — appliqué : M14 écrite avec sa propagation (`map_err(kesh_db::errors::map_db_error)?`).
    **F6** — appliqué : l'état attendu de l'AC7 décrit par contenu, numéros de `89d68704` en parenthèse. **F7** —
    appliqué : en-tête (A5, P2-2, B4 en partie) ; AC14 « 34 qui doivent rougir, et M8, équivalente déclarée ».
  - **Propagation** (grep des valeurs sur la fiche et le registre) : `:171`, `85-141`, `85-124`, `:529-539`,
    `deux de ses points`, `à ouvrir`, `toutes visibilités`, `limite qui reste`, `l: &Lecture` — traités dans les entrées
    vivantes ; les mentions des entrées « fiche rédigée » et « validation P1 » restent, relevés datés (la seule
    correction de la P1 est l'annotation datée de F3) ; `sprint-status.yaml` : annotation datée ajoutée à la ligne de la story.
  - **Suite** : la boucle ne se clôt que sur une passe à 0 MEDIUM. P3 **ciblée**, une lentille (regression hunter),
    **Sonnet**, sur le diff de cette remédiation et l'AC12 entier (`324a-validate-prompt-p3.md`).
- 2026-10-24 — **Validation P3, passe ciblée** (Sonnet, une lentille — regression hunter —, contexte frais, sur
  `git show cbcd3237` et l'AC12 entier ; prompt `324a-validate-prompt-p3.md`, rapport
  `kesh-gate-logs/324a-validate-p3.md`) : **0 CRITICAL, 0 HIGH, 0 MEDIUM, 8 LOW** — **validation terminée**. Elle
  confirme 15 / 10 / 12 occurrences de `state.pool` (fichier = somme des corps), la (ii) de `list_accounts` ramenée à
  la définition unique, le choix des corps par nom exact, M16′ tuée, 35 = 34 + M8, M14 compilable, un seul
  `FOR UPDATE` dans la fermeture (`journal_entries.rs:1182`, branche non prise). **Les huit LOW ne sont pas appliqués**
  — une remédiation écrite après la passe de clôture ne serait relue par personne — et **se recontrôlent à l'étape 0
  du développement**, ligne lue citée au Dev Agent Record :
  - **P3-1** : la requête de `find_source_document_of_entry` est aux l. 387-396 (non 386-395).
  - **P3-2** : la clause `contains("pub async fn")` de `services_motif.rs` est à la l. 2091 (non 2088-2090).
  - **P3-3** : le compte au niveau du fichier ne couvre **pas** le compte des `async fn` retiré en P2 — une `async fn`
    non `pub` qui lit le pool sous un autre nom rougissait en P1 et ne rougit plus ; à écrire aux limites de l'AC12 (ou
    à rétablir le compte, au choix du développement, au registre).
  - **P3-4** : la liste des limites de l'AC12 omet `self.pool` (un `impl AppState`), `(&state).pool`,
    `state.clone().pool` et une macro à paramètre ident.
  - **P3-5** : le *Hors périmètre* de B4 dit « la limite est écrite à l'AC12 » ; les autres formes de B4 ne sont qu'à
    l'AC17.
  - **P3-6** : M14 écrit `INSERT INTO audit_log …`, qui n'est pas du SQL : recopié tel quel, il rendrait 1064
    (syntaxe) et non 1792 — écrire une instruction complète et valide.
  - **P3-7** : la plage `services_motif.rs:43-141` emporte `BORNE_*` et `EXERCICE_*` (l. 46-52), inutiles au montage —
    `dead_code` sous `clippy -D warnings` si recopiées en bloc.
  - **P3-8** : le registre ne porte pas le choix de R3 (définition du corps retenue, contre la définition inverse que
    R3 préférait) — à ajouter au registre ; les deux doc-comments de `kesh-db` (`journal_entries.rs:775-776`, `:934`)
    sont tracés sur **#738** (commentaire du 2026-10-24).
  **Synthèse de la boucle** : P1 (Sonnet ×2, R/F) 0/0/3/15 → P2 (Opus ×2, R/F) 0/0/1/14 (le MEDIUM né de la remédiation
  P1) → P3 ciblée (Sonnet) 0/0/0/8. Pas de découpage : la sévérité décroît, et le MEDIUM de P2 venait du patch précédent
  (amendement D5). **Q1** reste posée à Guy (§ *Points à soumettre à Guy*) ; le développement part sur la première
  lecture. Statut : `ready-for-dev`.
- **2026-10-24 — développement** (Opus 5.5, en autonomie, `bmad-dev-story`) : commits `c550ea8c` (T1 (i), joué sur
  le code d'avant), `7abc9073` (T1 (ii), constaté non compilable avant T2), `90dbcf5b` (T2 à T5, code), `345bcc7f`
  (T5, doc-comments et document des motifs — **dernier commit de code**), puis ce compte rendu. Trois services neufs,
  cinq handlers minces, `Portee::ContactsRead`, garde de transaction déplacée, garde lexicale étendue ((ii), (iii),
  (viii)), dix-neuvième ligne de la table des traces. Étape 0 : P3-1 à P3-8 recontrôlés et appliqués (P3-3 par le
  compte des `async fn` rétabli, C-32-4a-13). Tests : **+12** de `39939799` à `HEAD` (`services_rollout.rs` 0 → 9,
  `services_motif.rs` 20 → 23), **1 déplacé** (`routes/journal_entries.rs` 8 → 7, `services/ecritures.rs` 0 → 1 ;
  `grep -cE '#\[(sqlx::|tokio::)?test'` aux deux bornes). Mutations : **34 tuées sur 35, M8 équivalente**. Gates à
  `345bcc7f` : ciblé 984/984 (1 ignoré) ; complet 4013/4013 (7 ignorés) ; frontend 1911 tests ; E2E 315 passés, 7
  échecs = les sept KF-029. Choix C-32-4a-13 à 18 au registre. Écart : un ajout de tests par `cat >>` (consigne 14),
  déclaré. **Q1** reste à Guy. Statut : `review` ; revue de code P1 à lancer (`324a-review-prompt-p1.md`, Sonnet,
  lentilles B, E, A).
