# Story 32.4a : la couche de services — rollout des lectures : écritures comptables, postes ouverts, contacts

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-4a`, branche `story/32-4a` partie de `main`
`89d68704` (la 32-3 fusionnée). Registre : `epic-32-choix-autonomes-324a.md` (C-32-4a-1 à C-32-4a-12). Clé courte :
`324a`. Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs` ; issue des LOW de la 32-3 : #738, en
`refs` (deux de ses points repris, C-32-4a-10). Les numéros de ligne cités sont ceux de `89d68704`.*

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
   la route (`tests/open_items_e2e.rs:274-323`, `:529-539`) ; celles des contacts et des écritures, par **aucun** test
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
   'get_or_create\|audit' routes/{contacts,letterings,journal_entries}.rs` → commentaires seuls) ; les dépôts appelés —
   `journal_entries::{list_by_company_paginated, find_by_id, reversal_blocker, reversal_blockers, reversed_by,
   modification_blocker}`, `imported_supplier_invoices::find_source_document_of_entry` (« lecture **ordinaire** (jamais
   `FOR UPDATE`) », `imported_supplier_invoices.rs:171`), `letterings::open_items` (sa transaction, `conn.begin()` l. 354, refermée par
   `rollback` l. 405 de `open_items.rs`), `contacts::{list_by_company_paginated, find_by_id_in_company}`,
   `helpers::company_for_ids` → `companies::find_by_id` — ne portent, sur ces chemins, ni écriture ni `FOR UPDATE`
   (`modification_blocker`, « Lecture **sans verrou** », `journal_entries.rs:1205`) : **établi par lecture statique**,
   à confirmer à l'exécution par l'AC11.
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
   (`Internal` si la société manque), `find_by_id(lecture.pool(), company.id, id)` (`NotFound` → 404), puis **une**
   transaction `lecture.pool().begin()` portant les quatre lecteurs du fait 5, et `rollback`. Rend
   `DetailEcriture` (champs `pub` : l'écriture et ses lignes, l'écriture qui la contre-passe, le motif de blocage de la
   contre-passation, le motif de blocage de la modification, la pièce source — noms au développement). La garde du fait 5
   **se déplace** au `#[cfg(test)] mod tests` de `services/ecritures.rs` (C-32-4a-7) : même nom
   (`get_journal_entry_reads_in_one_transaction`, ou `detail_lit_dans_une_transaction` avec renvoi à la 15-1b-0 au
   doc-comment — au choix, écrit au Dev Agent Record), mêmes assertions sur le corps de `pub async fn detail(` lu par
   `include_str!("ecritures.rs")` tronqué à `#[cfg(test)]` : `pool.begin()` une fois, `acquire()` zéro fois, chacun des
   quatre lecteurs appelé une fois sur `&mut tx` ou `&mut *tx` et non sur le pool, et l'écart assumé
   `journal_entries::find_by_id(lecture.pool()` ; le refus d'un lecteur sur le pool cherche `.pool()` au lieu de
   `state.pool`. Le diff entre l'ancienne et la nouvelle garde est recopié au Dev Agent Record.
5. **AC5 — `postes_ouverts::lister`.** Reçoit `(&ContexteAcces, &Lecture, DemandePostesOuverts)`. Après `exiger` :
   `parse_as_of` (déplacée telle quelle de `letterings.rs:503-527`, doc-comment compris, `Utc::now().naive_utc().date()`
   si absente ou vide), puis défaut (`LIMITE_DEFAUT` 50) et `clamp(1, LIMITE_MAX)`, `offset` défaut 0 puis `max(0)`,
   puis `company_for_ids`, puis une connexion acquise de la `Lecture` et `letterings::open_items(&mut conn, company.id,
   account_id, as_of, limit, offset)`. **Ordre des refus inchangé** (AC7 de la 15-1b, doc-comment l. 530-533) : `asOf`
   (400) avant toute lecture, compte introuvable ou d'une autre société (404), compte non lettrable (409).
   `LIMITE_MAX` est **le plafond commun** : `get_lettering_proposals`, qui reste au handler, l'importe du service au lieu
   de son `MAX_LIMIT` local, qui disparaît (une seule constante, C-32-4a-5) ; `PROPOSALS_DEFAULT_LIMIT` reste au handler.
6. **AC6 — `carnet::lister` et `carnet::detail`.** `lister(&ContexteAcces, &Lecture, DemandeListeContacts)` : après
   `exiger`, `company_for_ids` **d'abord** (fait 7), puis défaut (`LIMITE_DEFAUT` 20) et `clamp(1, LIMITE_MAX)` (100),
   `offset` défaut 0 puis `max(0)`, puis `contacts::list_by_company_paginated(lecture.pool(), acces.company_id(),
   query)` ; rend `PageContacts { page, societe }`. `detail(&ContexteAcces, &Lecture, id)` : après `exiger`,
   `find_by_id_in_company(lecture.pool(), id, acces.company_id())` → `NotFound` **avant** `company_for_ids` (fait 7) ;
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
   du service (ou rien), recopié au Dev Agent Record. Les mutants, `get_lettering` et `get_lettering_proposals` ne
   changent pas, hors l'import du plafond commun (AC5).

### C — Les tests

8. **AC8 — les tests existants restent verts ; les seules modifications sont celles-ci.** (1) `tests/services_motif.rs` :
   `TOUTES` à trois portées (l. 869) et un bras `ContactsRead` à `autre_portee` (l. 894-899) (C-32-4a-4) ; le compte de
   `services/` à **huit** fichiers (l. 2137-2142) ; la garde étendue de l'AC12 ; `pool_lecture_seule` remplacée par
   l'aide commune ; la ligne 7 de `TABLE_DES_TRACES` (l. 432-442) à `journal: None`, `nom` et `details` en conséquence
   (`"journal_filter": null`) (AC17). (2) `tests/common/mod.rs` : `pub mod lecture_seule;` ; `tests/common/lecture_seule.rs`
   neuf (C-32-4a-9). (3) `routes/journal_entries.rs` : le `mod tests` perd la seule garde du fait 5 (l. 1005-1080),
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
   écriture sur un compte lettrable — établi au code des fixtures, sinon posé en SQL —, un contact). Une assertion nommée
   par cas (C-32-3-38), pour **chacun des cinq services** :
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
    - (c) **par la route** (montage HTTP comme `services_motif.rs:85-124`) : `GET /api/v1/contacts?limit=1000&offset=-3`
      → `limit` 100, `offset` 0 ; `GET /api/v1/contacts` → `limit` 20 ; `GET /api/v1/journal-entries?limit=0` →
      `limit` 1 ; `GET /api/v1/journal-entries` → `limit` 50 ; `GET /api/v1/journal-entries?dateFrom=x` → 400, code
      `VALIDATION_ERROR` et message de la table. Les bornes des postes ouverts par la route sont déjà figées
      (`open_items_e2e.rs:274-323`, `:529-539`), non recopiées.
11. **AC11 — lecture seule, et filtre de société.** Avec l'aide commune `lecture_seule` (pool dont chaque connexion
    passe `SET SESSION TRANSACTION READ ONLY`, C-32-4a-9) :
    - (a) les **cinq** services **réussissent** sur cette `Lecture` (le détail d'écriture compris : `BEGIN`, quatre
      lectures, `ROLLBACK` ; les postes ouverts : leur transaction propre) — confirmation, **sur les branches que les
      fixtures exercent**, de l'inventaire de l'AC13 ; les branches non atteintes (pièce source présente, motifs de
      blocage présents…) sont **nommées** au Dev Agent Record, non revendiquées ;
    - (b) **filtre de société** : une seconde société (fixture synthétique) ; un contexte MCP de cette société, toutes
      portées, demande le détail de l'écriture et du contact de la première → `NotFound` ; les postes ouverts du compte
      de la première → `NotFound` ; `ecritures::lister` et `carnet::lister` → aucune ligne de la première.
12. **AC12 — la garde lexicale, étendue (R8), dans `services_motif.rs`.** Mêmes définitions que l'AC10 de la 32-3
    (lignes de code, corps de handler de sa ligne d'ouverture au `pub async fn` suivant ou au `#[cfg(test)]`) :
    - (ii) étendue : les corps des **six** handlers `accounts::list_accounts`, `journal_entries::{list_journal_entries,
      get_journal_entry}`, `letterings::get_open_items`, `contacts::{list_contacts, get_contact}` ne contiennent pas
      `state.pool` ; chaque corps est non vide (plancher de lignes) ; **témoin positif** (#738 A5) : le même détecteur,
      appliqué à un texte témoin **et** au corps réel de `letterings::get_lettering`, y trouve `state.pool` ;
    - (iii) **inventaire des sites non résolus** : dans les trois fichiers visés, l'ensemble des couples (fichier,
      handler) dont le corps contient `state.pool` est **exactement** les douze attendus — les dix mutants
      (`journal_entries::{reverse_journal_entry, reverse_journal_entry_at_next_fiscal_year_start, create_journal_entry,
      update_journal_entry, delete_journal_entry}`, `letterings::{create_lettering, delete_lettering}`,
      `contacts::{create_contact, update_contact, archive_contact}`) et `letterings::{get_lettering,
      get_lettering_proposals}` (C-32-4a-1) — et le nombre de `pub async fn` par fichier vaut 7 / 5 / 5 ; un handler
      neuf sur `state.pool` y rougit par son nom ;
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
    étendu) et au Dev Agent Record : (1) **aucune** écriture des services : les lecteurs du fait 6 ne font que lire —
    établi par lecture statique (le développement relance `grep -nE "INSERT|UPDATE |DELETE|FOR UPDATE|get_or_create"`
    sur le corps de chacun, sortie recopiée) et confirmé par l'AC11 (a) ; (2) `api_keys::touch_last_used`, écrite par
    le **middleware** pour une requête par clé d'API — hors service, inchangée (point 2 de l'inventaire de la 32-3).
14. **AC14 — mutations qui doivent rougir** (une à la fois, restauration `git checkout -- <f> && touch <f>`, diff
    résiduel nul ; gate ciblé de l'AC15 ; chacune rougit **au moins une assertion nommée**, recopiée au Dev Agent
    Record avec son diff) — **33** :
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
    - **M14** — une écriture sur la `Lecture` (un `INSERT INTO audit_log` par `lecture.pool()` dans `carnet::detail`) →
      AC11 (a) (1792) ;
    - **M15** — `get_contact` relit le contact par `state.pool` → AC12 (ii) et (iii) (#738 A5 : la garde (ii) rougit
      enfin sous une mutation) ;
    - **M16** — un handler de lecture neuf sur `state.pool` dans `letterings.rs` → AC12 (iii) ;
    - **M17** — le service `rapports::journaux` trace un filtre **absent** comme un filtre présent (le premier journal) →
      ligne 7 de `TABLE_DES_TRACES` (AC17 ; aucune ligne ne le voyait avant) ;
    - **M18** — un paramètre `trace: &Trace` ajouté à un service du rollout → AC12 (viii).
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
    déplacé (défauts et bornes des postes ouverts et des propositions identiques) ; `grep -nE "contacts|journal-entries|open-items" docs/manual/fr/*.tex`
    et les PDF aplatis (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `grep -oiE
    ".{60}(par page|pagination|100 contacts|500 écritures).{60}"`) : aucun texte ne décrit autrement les bornes. **Deux
    textes bougent** : la note de `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (l. 273-276) nomme les cinq lectures migrées
    et dit que la liste des écritures ne lit pas la société (C-32-4a-6) ; le doc-comment de `services/mod.rs` (AC1,
    AC13). Aucun écran ne change : recette visuelle sans objet. Les doc-comments de module de
    `routes/{journal_entries,letterings,contacts}.rs` renvoient en une ligne aux services pour les lectures migrées.
17. **AC17 — deux points de #738 repris** (C-32-4a-10) : **A5** (AC12 (ii), témoin et M15) ; **P2-2** (ligne 7 de
    `TABLE_DES_TRACES` sans filtre, M17). Le corps de la PR les nomme ; #738 reste ouverte (`refs`) pour les autres.

## Tâches

- [ ] **T0** — Worktree et branche (posés) ; bases `kesh_324a`, `kesh_e2e_324a` créées, migrées depuis les migrations
  du worktree, seedées ; port attribué par l'orchestrateur ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-32-4a/target` ;
  `node_modules` par `cp -al` (`docs/agents-orchestration.md` § 4). Étape 0 : recontrôler au code les lignes citées des
  faits 1 à 9 (une ligne lue citée par fait au Dev Agent Record).
- [ ] **T1 (AC9-AC11 — rouge d'abord)** — `tests/common/lecture_seule.rs`, `tests/services_rollout.rs` : en **deux
  commits** — (i) AC10 (c) **seul** (par la route, sans référence aux services neufs), **joué vert sur le code d'avant
  T2** : c'est le relevé qui établit que la table des bornes et des refus décrit le code d'aujourd'hui ; (ii) le reste,
  dont on constate qu'il ne compile pas avant T2.
- [ ] **T2 (AC1-AC4, AC7)** — `Portee::ContactsRead` ; `services/ecritures.rs` (`lister`, `detail`, la garde déplacée) ;
  handlers minces ; gate ciblé.
- [ ] **T3 (AC5, AC7)** — `services/postes_ouverts.rs` ; `get_open_items` mince ; plafond commun importé par
  `get_lettering_proposals` ; gate ciblé.
- [ ] **T4 (AC6, AC7)** — `services/carnet.rs` ; `list_contacts`, `get_contact` minces ; gate ciblé.
- [ ] **T5 (AC8, AC12, AC17)** — amendements de `services_motif.rs` (liste fermée de l'AC8 (1)) ; doc-comment de
  `services/mod.rs` (AC1, AC13) ; textes de l'AC16.
- [ ] **T6 (AC14)** — mutations M1a à M18, chacune tuée ou déclarée (M8 équivalente).
- [ ] **T7 (AC8, AC13, AC15, AC16)** — contrôles de l'AC8, gate complet et E2E complet au dernier commit de code ; Dev
  Agent Record : chiffres réellement exécutés, listes exactes, inventaire.

## Dev Notes

### Sites

- `crates/kesh-api/src/routes/journal_entries.rs` — constantes l. 41-50 ; DTO l. 291-330 ; `list_journal_entries`
  l. 345-487 ; `get_journal_entry` l. 489-532 ; `mod tests` l. 855, garde l. 1005-1080.
- `crates/kesh-api/src/routes/letterings.rs` — constantes l. 286-300 ; `OpenItemsQuery` l. 302-312 ; `parse_as_of`
  l. 503-527 ; `get_open_items` l. 530-551 ; `get_lettering` (l. 236) et `get_lettering_proposals` (l. 557), qui
  restent. Aucun `#[cfg(test)]`.
- `crates/kesh-api/src/routes/contacts.rs` — constantes l. 42-43 ; `ListContactsQuery` l. 48-70 ;
  `contact_response_with_label` l. 510-522 ; `list_contacts` l. 563-602 ; `get_contact` l. 607-622 ; `mod tests` l. 784.
- `crates/kesh-api/src/services/{mod,acces}.rs` — liste des modules (l. 102-105), inventaire (l. 76-89) ; `Portee`.
- `crates/kesh-api/tests/services_motif.rs` — l. 869, 894-899, 835-853, 2073-2100, 2131-2151, 432-442.
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
pub async fn lister(a: &ContexteAcces, l: &Lecture, d: DemandeListeEcritures) -> Result<JournalEntryListResult, AppError>;
pub struct DetailEcriture { /* écriture, contre-passée par, blocage contre-passation, blocage modification, pièce */ }
pub async fn detail(a: &ContexteAcces, l: &Lecture, id: i64) -> Result<DetailEcriture, AppError>;
// services/postes_ouverts.rs
pub struct DemandePostesOuverts { pub account_id: i64, pub as_of: Option<String>, pub limit: Option<i64>, pub offset: Option<i64> }
pub async fn lister(a: &ContexteAcces, l: &Lecture, d: DemandePostesOuverts) -> Result<OpenItems, AppError>;
// services/carnet.rs
pub struct PageContacts { pub page: ContactListResult, pub societe: Company }
pub struct FicheContact { pub contact: Contact, pub societe: Company }
pub async fn lister(a: &ContexteAcces, l: &Lecture, d: DemandeListeContacts) -> Result<PageContacts, AppError>;
pub async fn detail(a: &ContexteAcces, l: &Lecture, id: i64) -> Result<FicheContact, AppError>;
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
  `Rapport` (32-9a) ; **B4** (gardes non étanches) — aucun faux vert aujourd'hui, et la limite est écrite à la garde ;
  **E3**, **E4** (ordre des rapports de projet, trace en échec) — tests des rapports, hors des modules visés ; **B5**
  (doc-comment déplacé dans `routes/reports.rs`) — un sixième fichier de production pour une ligne de commentaire ;
  **A3**, **A4**, **P2-1** (textes de la fiche 32-3) — une fiche close est un relevé daté, non une page à tenir ; **A6**
  — non vérifiable. #738 reste ouverte pour eux.
- **La lecture seule de production** (Q11) — 32-6, comme pour la 32-3.

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

### Références

`CLAUDE.md` § *Test Locally First*, § *Pendant une boucle de revue*, § *Règle de splitting préventif*, § *Inventorier
les sites NON RÉSOLUS*, § *Recompter ses propres comptes rendus* ; `docs/agents-orchestration.md` § 2-7 ;
`planning-artifacts/epic-32-serveur-mcp.md` § *32-4*, § *32-9* ; fiche `32-3-services-motif.md` (§ *Le motif*, AC7,
AC8, AC10, R1-R8) ; registres `epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7) et `-323.md` (C-32-3-4, C-32-3-36 à
41) ; #738.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- **2026-10-24 — fiche rédigée** (Opus 5.5, en autonomie, worktree `kesh-32-4a`, base `89d68704`) : 17 AC, 33
  mutations nommées (M8 déclarée équivalente d'avance), 4 modules (non découpée) ; registre
  `epic-32-choix-autonomes-324a.md`, C-32-4a-1 à 12 ; deux points de #738 repris (A5, P2-2). Validation P1 à lancer
  (`324a-validate-prompt-p1.md`, Sonnet, lentilles R et F).
