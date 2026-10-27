//! Couche de services — la lecture des livres, partagée par l'API et le futur
//! serveur MCP (Epic 32, Story 32-3).
//!
//! Un service est le **seul** lieu où une lecture des livres décide qui a le
//! droit de lire, valide la demande, lit, et prépare la trace d'audit. Le
//! handler HTTP (`routes/…`) n'en garde que la présentation ; le MCP (32-6)
//! appellera les mêmes fonctions, sans passer par Axum.
//!
//! # Le motif
//!
//! ```text
//! handler (routes/…)            service (services/…)                        poignées
//!   extraire la requête   ──►   1. acces.exiger(Exigence)        ◄── avant toute lecture
//!   (format d'export :          2. valider la demande (règles d'aujourd'hui, même ordre)
//!    validé ici, présentation)  3. lire par Lecture seulement    ──► Lecture  (lecture des livres)
//!                               4. rendre Rapport<T, U>
//!   rapport.afficher(&trace) / rapport.exporter(&trace, rendre)  ──► Trace    (journal d'audit)
//!   mettre en forme la réponse HTTP
//! ```
//!
//! - `acces::ContexteAcces` dit **qui** lit (société, utilisateur, canal `Api`
//!   ou `Mcp`) ; `acces::Exigence` dit ce qu'une opération exige (un rôle pour
//!   l'API, une portée — ou `None`, jamais servi — pour le MCP).
//! - `poignees::Lecture` et `poignees::Trace` sont deux poignées **distinctes**
//!   sur la base : l'une pour lire les livres, l'autre pour écrire la trace.
//!   En production, elles ne se fabriquent que par `AppState::lecture()` et
//!   `AppState::trace()`.
//! - `rapports::Rapport` est ce que rend un service de rapport : son contenu ne
//!   s'obtient que par la méthode qui écrit la trace.
//!
//! # Les règles (pour le rollout de la 32-4)
//!
//! - **R1** — Un service ne reçoit ni `AppState`, ni type d'Axum :
//!   `(&ContexteAcces, &Lecture, demande[, usage])`, plus `&Trace` là où il
//!   trace. Le MCP l'appellera tel quel.
//! - **R2** — Première instruction : `acces.exiger(EXIGENCE)?` — avant la
//!   validation, avant toute lecture.
//! - **R3** — Une `Exigence` par opération, constante nommée à côté du service ;
//!   pour l'API, le rôle **égal** au RBAC de la route (le middleware reste ; le
//!   service ne l'assouplit jamais) — et, pour une aide servie par **plusieurs**
//!   routes de RBAC différents, **le plus bas** de leurs rôles, chaque route
//!   gardant le sien par son propre service et son middleware
//!   (`rapports::identite_societe`) ; pour le MCP, une portée, ou `None`.
//! - **R4** — Toute lecture passe par la `Lecture` ; **aucune écriture** sur un
//!   chemin de lecture : ce qui doit écrire (trace, dernière utilisation) passe
//!   par la `Trace`. Le pool d'une poignée ne sort pas de `services`
//!   (`pub(in crate::services)`, champ privé — le compilateur le tient ; aucun
//!   `unsafe` dans `services`, `forbid(unsafe_code)` en tête de ce module) ;
//!   **dans** `services`, on n'ouvre aucun autre chemin vers lui et on n'emploie
//!   `lecture.pool()` qu'à lire, `trace.pool()` qu'aux fonctions d'audit — ce
//!   qu'aucune garde ne prouve : la revue le lit, le test
//!   `tests/services_motif.rs` le vérifie à l'exécution, et la connexion des
//!   outils du MCP se déclare en outre `READ ONLY` (mode de session, 32-6a ;
//!   Q11, tranchée le 2026-10-24). Ce qu'un handler doit lire pour sa présentation
//!   (le contexte PDF) passe par une fonction de service. L'action d'audit reste
//!   un **littéral** à chaque `NewAuditLogEntry::user(` (garde
//!   `tests/audit_label_registry.rs`). Les envois du serveur MCP (Story 32-8b)
//!   ont leurs poignées propres, sans pool visible : `LectureEnvois` (une
//!   relecture, sur le pool en lecture seule du MCP) et `EcritureEnvois` (le
//!   dépôt avec l'entrée d'audit de son appel, et la purge des contenus échus,
//!   sur le pool ordinaire) — les deux seules écritures d'un chemin du MCP hors
//!   de la `Trace` (inventaire ci-dessous, point 2).
//! - **R5** — Une trace qu'un handler pourrait oublier est une trace muette : le
//!   contenu ne se rend qu'à travers la méthode qui trace
//!   (`Rapport::afficher`/`exporter`). Une lecture sans trace (le plan
//!   comptable) rend directement.
//! - **R6** — La validation de la demande vit dans le service, **dans l'ordre
//!   d'aujourd'hui** ; la présentation (format d'export, rendu, en-têtes, nom de
//!   fichier) reste au handler.
//! - **R7** — Chaque service a son test de refus (portée absente, canal refusé,
//!   refus avant la validation **et** avant la lecture) **et** son contrôle
//!   positif ; un test qui l'exécute sur une `Lecture` en lecture seule ; et,
//!   s'il trace, sa ligne dans la table des traces, au service et par la route.
//! - **R8** — La garde lexicale de `tests/services_motif.rs` s'étend aux
//!   handlers migrés ; la liste des fabriques de poignées s'amende en
//!   connaissance de cause, ligne par ligne, jamais par un motif élargi. Elle ne
//!   prouve pas l'inaccessibilité du pool : la revue de chaque rollout lit chaque
//!   appel de `pool` de `services/` (`.pool(`, `::pool`). Un module de services
//!   ajouté par la 32-4 hérite du `forbid(unsafe_code)` de ce module, qu'il ne
//!   doit pas déplacer.
//!
//! # Inventaire clos des écritures sur les chemins de lecture des pilotes
//!
//! 1. Les **sept** fonctions d'audit des rapports (`rapports.rs`), dix-huit
//!    sites d'appel — un par handler de rapport —, toutes sur la `Trace`.
//! 2. `api_keys::touch_last_used` (`auth/api_key.rs`), écrite par le
//!    **middleware** sur `AppState::pool` pour une requête par clé d'API — hors
//!    service, inchangée. La clé MCP écrit la sienne
//!    (`mcp_keys::touch_last_used`) sur la `Trace`, dans
//!    `cles_mcp::authentifier` (Story 32-5b) — une écriture d'authentification,
//!    non d'un chemin de lecture des livres.
//!
//!    Et, depuis la Story 32-6b, `serveur_mcp::journaliser_appel` : l'entrée
//!    `mcp.tool_called` de chaque appel d'outil MCP, sur la `Trace`, écrite par
//!    le répartiteur du MCP avant l'envoi de la réponse — **fail closed** : si
//!    elle échoue, aucune donnée ne part (C-32-6-30). Dérogation écrite aux
//!    règles R1, R2 et R7 (une écriture d'audit, sans `exiger`).
//!
//!    Et, depuis la Story 32-8b, deux écritures des **envois du serveur MCP**,
//!    sur la poignée `poignees::EcritureEnvois` (pool ordinaire), jamais sur une
//!    `Lecture` : (a) `envois_mcp::deposer` — le dépôt d'un envoi **et**
//!    l'entrée `mcp.tool_called` de son appel (`serveur_mcp::entree_appel`), en
//!    **une** transaction ; (b) la purge des contenus échus de la société
//!    (`EcritureEnvois::purger_echus`), *best-effort*, hors de cette
//!    transaction. Dérogation écrite aux règles R2 et R7 (`envois_mcp`, en tête
//!    du module) : la portée a été exigée par le service de l'outil avant sa
//!    lecture, et le dépôt ne lit rien des livres. La relecture
//!    (`envois_mcp::lire`, sur `poignees::LectureEnvois`, pool en lecture seule
//!    du MCP) déroge de même à R2 et R7 : sa règle est « même clé, même
//!    société », plus étroite qu'une portée.
//!
//!    La lecture des **prompts du serveur MCP** (Story 32-10a-ii,
//!    `prompts_mcp::{lister, obtenir}`) passe par sa poignée propre,
//!    `poignees::LecturePrompts`, sur le pool en lecture seule du MCP : une
//!    lecture, aucune écriture, qui suit R2 et R7.
//! 3. Aucune autre : les générateurs de `kesh-report`, `ReportPeriod::resolve`,
//!    `resolve_scope`, `rapports::identite_societe`, `helpers::company_for_ids`,
//!    `accounts::list_by_company` et `letterings::letterable_account_ids` ne
//!    font que lire — établi par la lecture du code, et confirmé, sur les
//!    branches que ses fixtures exercent, par `tests/services_motif.rs`, qui les
//!    exécute sur une connexion en lecture seule.
//!
//!    **Étendu par la 32-4a** aux cinq services du rollout : sur la fermeture
//!    de leurs appels — appels directs (`journal_entries::{list_by_company_paginated,
//!    find_by_id, reversal_blocker, reversed_by, modification_blocker}`,
//!    `imported_supplier_invoices::find_source_document_of_entry`,
//!    `letterings::open_items`, `contacts::{list_by_company_paginated,
//!    find_by_id_in_company}`, `companies::find_by_id`) **et** transitifs,
//!    textes SQL en `const` compris —, aucune écriture ni `get_or_create`, et
//!    un seul `FOR UPDATE` : la branche `Lecture::Verrouillante` de
//!    `journal_entries::lettering_guard`, **non prise** (`modification_blocker`
//!    passe `Lecture::Conseil`). Établi par lecture statique (relevé de
//!    l'AC13 de la 32-4a, sur le code des fichiers de dépôts tronqué à leurs
//!    tests) ; confirmé à l'exécution, sur les branches que ses fixtures
//!    exercent, par `tests/services_rollout.rs` (AC11 (a)), qui les exécute sur
//!    une connexion en `SET SESSION TRANSACTION READ ONLY` — où MariaDB refuse
//!    l'écriture **et** le `SELECT … FOR UPDATE`, en autocommit comme dans une
//!    transaction (erreur 1792, mesuré et figé par l'AC11 (c)). Branches non
//!    atteintes par ces fixtures : pièce source présente, motifs de blocage de
//!    la contre-passation et de la modification présents, compte non lettrable
//!    (refusé par `require_viewable_account` avant toute lecture des postes) et
//!    contact portant des conditions de paiement (leur libellé est mis en forme
//!    au handler, hors service).
//!
//!    **Étendu par la 32-4b** aux six services des factures : sur la fermeture
//!    de leurs appels — appels directs (`invoices::{list_by_company_paginated,
//!    find_by_id_with_lines, due_dates_summary}`,
//!    `invoice_settlements::{amount_settled, amount_due, list_for_invoice}`,
//!    `invoice_settlements_write::settlement_cancel_blocker`,
//!    `company_invoice_settings::round_to_5_centimes` — une lecture, **pas** le
//!    `get_or_create` du réglage —, `supplier_invoices::{list,
//!    get_settlement_view}`, `companies::find_by_id`) **et** transitifs
//!    (`settlement_cancel_blocker_unlinking`,
//!    `settlement_cancellation::settlement_entry_cancel_blocker`,
//!    `letterings::{document_group_frozen_by_periods, lines_in_open_period,
//!    open_period_rule, books_locked_through}`,
//!    `journal_entries::{reversal_blocker, reversal_blockers, reversed_by,
//!    document_owners}`, `fiscal_years::{has_open_covering_date,
//!    find_later_closed}`, `supplier_invoices::{fetch_lines,
//!    supplier_settlement_cancel_blocker, supplier_invoice_cancel_blocker,
//!    in_generated_batch}`, `payment_batches::last_confirmed_batch_for_invoice`,
//!    `imported_supplier_invoices::find_by_supplier_invoice_id_scoped`, les aides
//!    pures de `util::search` et `invoices::push_where_clauses`), textes SQL en
//!    `const` et les deux macros SQL (`invoice_columns!`, `line_ttc_sql!`)
//!    compris — cinquante-neuf éléments —, **aucune** écriture, aucun
//!    `get_or_create`, aucun `FOR UPDATE` ni verrou partagé. Établi par lecture
//!    statique (relevé de l'AC14 de la 32-4b, sur le code des fichiers de
//!    dépôts tronqué à leurs tests, commentaires écartés) ; confirmé à
//!    l'exécution, sur les branches que ses fixtures exercent, par
//!    `tests/services_rollout.rs` (`ac11_a_les_services_lisent_en_lecture_seule`)
//!    sur la même connexion en lecture seule — un brouillon (le réglage lu),
//!    une facture validée portant un règlement (la transaction des règlements,
//!    son `commit` et `settlement_cancel_blocker` exécutés), une facture
//!    fournisseur (la transaction de `get_settlement_view` et son `commit`).
//!    Branches non atteintes par ces fixtures : motifs de refus d'annulation
//!    présents, pièce source et lot confirmé présents, facture fournisseur
//!    réglée, et les variantes de filtre des listes (recherche, statut de
//!    paiement, suspension, tri) — des clauses de `SELECT`, lues ailleurs par
//!    la route.
//!
//! # Le rollout de la 32-4a — cinq lectures, sans trace
//!
//! | Service | Route | Exigence API / MCP |
//! |---|---|---|
//! | `ecritures::lister` | `GET /journal-entries` | `Consultation` / `BooksRead` |
//! | `ecritures::detail` | `GET /journal-entries/{id}` | `Consultation` / `BooksRead` |
//! | `postes_ouverts::lister` | `GET /accounts/{id}/open-items` | `Consultation` / `BooksRead` |
//! | `carnet::lister` | `GET /contacts` | `Consultation` / `ContactsRead` |
//! | `carnet::detail` | `GET /contacts/{id}` | `Consultation` / `ContactsRead` |
//!
//! Chacun porte ses bornes de pagination et la validation de sa demande
//! (C-32-4a-5) : le MCP n'a pas le DTO d'Axum. Restent dans leur handler, sur
//! `state.pool`, figées par la garde (iii) de `tests/services_motif.rs` : les
//! dix mutations de `routes/{journal_entries,letterings,contacts}.rs`
//! (C-32-0-4, 0.17.0), et deux lectures qu'aucun outil MCP prévu ne sert —
//! `letterings::get_lettering` (`GET /letterings/{key}`) et
//! `letterings::get_lettering_proposals` (`GET /accounts/{id}/lettering-proposals`),
//! C-32-4a-1.
//!
//! # Le rollout de la 32-4b — six lectures des factures, sans trace
//!
//! | Service | Route | Exigence API / MCP |
//! |---|---|---|
//! | `factures::lister` | `GET /invoices` | `Consultation` / `InvoicesRead` |
//! | `factures::detail` | `GET /invoices/{id}` | `Consultation` / `InvoicesRead` |
//! | `factures::echeances` | `GET /invoices/due-dates` | `Consultation` / `InvoicesRead` |
//! | `factures::reglements` | `GET /invoices/{id}/settlements` | `Consultation` / `InvoicesRead` |
//! | `factures_fournisseur::lister` | `GET /supplier-invoices` | `Consultation` / `InvoicesRead` |
//! | `factures_fournisseur::detail` | `GET /supplier-invoices/{id}` | `Consultation` / `InvoicesRead` |
//!
//! Chacun porte ses bornes et sa validation (les listes clientes **refusent**
//! une taille de page hors bornes, la liste fournisseur l'écrête —
//! C-32-4b-6) ; `factures::detail` et `factures::echeances` ne lisent pas la
//! société (C-32-4b-8). Restent dans leur handler, sur `state.pool`, figés par
//! la garde (iii) : les douze mutations de `routes/{invoices,supplier_invoices}.rs`
//! (C-32-0-4) et leurs deux aides de `invoices.rs`
//! (`with_draft_rounding_preview`, `ensure_contact_belongs_to_company`) ;
//! l'export CSV de l'échéancier (`GET /invoices/due-dates/export.csv`,
//! Comptable+), qu'aucun outil ne sert et qui n'emploie du service que la
//! fonction pure `factures::requete_echeances` (C-32-4b-1, C-32-4b-4) ; et
//! `scan_qr_supplier_invoice`, qui ne lit pas la base. La balance âgée des
//! débiteurs (`GET /reports/aged-receivables`) est **déjà** un service
//! (`rapports::balance_agee`, 32-3) : l'outil de la 32-9a l'appellera tel
//! quel.
//!
//! # Le rollout de la 32-9a — la liste des exercices, sans trace
//!
//! | Service | Route | Exigence API / MCP |
//! |---|---|---|
//! | `exercices::lister` | `GET /fiscal-years` | `Consultation` / `BooksRead` |
//!
//! Sans lecture de la société ni borne propre : le comportement de la route
//! est conservé (le dépôt filtre par société et borne la liste). L'inventaire
//! clos des écritures sur les chemins de lecture s'y étend :
//! `fiscal_years::list_by_company` ne fait qu'un `SELECT` (établi par lecture).
//! Les autres gestionnaires de `routes/fiscal_years.rs` restent sur
//! `state.pool`, figés par la garde (iii). La 32-9a ajoute aussi
//! `factures_fournisseur::detail_sans_fichier`, qui appelle `detail` et ne
//! rend de la pièce que son identifiant : aucun chemin de fichier n'atteint
//! `src/mcp/` (C-32-9-13) ; et, à la revue de code P1 (A1, C-32-9-49),
//! `factures::detail_sans_fichier`, qui projette la facture cliente sans son
//! PDF figé (`pdf_storage_path` et ses trois compagnons).
//!
//! # Le rollout de la 32-9b — la liste des pièces importées, sans trace
//!
//! | Service | Route | Exigence API / MCP |
//! |---|---|---|
//! | `pieces::lister_tout` | `GET /imported-supplier-invoices?status=` | `Comptable` / `DocumentsRead` |
//! | `pieces::lister` | (l'outil MCP `kesh_pieces` seul) | `Comptable` / `DocumentsRead` |
//!
//! L'ordre de la route est conservé : la société d'abord (`Internal` si elle
//! manque), puis la validation du statut, puis la lecture
//! (`imported_supplier_invoices::list_by_status`, un `SELECT`, établi par
//! lecture). `lister` tranche la liste et la projette en `PiecePublique`, qui
//! ne porte ni l'empreinte ni le chemin du fichier, ni aucune donnée lue du
//! QR-code : aucun chemin de fichier n'atteint `src/mcp/` (C-32-9-13). Les
//! autres gestionnaires de `routes/imported_supplier_invoices.rs` — l'import,
//! la complétion, l'écart, les fournisseurs proposés, les documents — restent
//! sur `state.pool`, figés par la garde (iii) : aucun outil ne les sert.
//!
//! # La lecture seule du MCP — Q11, tranchée le 2026-10-24
//!
//! La réponse de Guy à Q11 : la lecture seule est **tenue par l'application**
//! — les outils MCP n'atteignent que des services qui reçoivent une `Lecture`
//! —, et la connexion des outils du MCP se déclare en outre `READ ONLY` (mode
//! de session) ; **aucun second compte MariaDB**. Réalisé par la 32-6a : les
//! outils lisent sur une `Lecture` construite au câblage du MCP
//! (`build_router_avec_registre`) sur `kesh_db::pool::pool_lecture_seule`, dont
//! chaque connexion passe `SET SESSION TRANSACTION READ ONLY` et est fermée à
//! sa restitution ; `AppState::lecture()` reste la `Lecture` **ordinaire** —
//! celle de l'API, et la `garde` du MCP (règle « servi », authentification,
//! C-32-6-38). La garantie repose sur le code de Kesh et ses tests (la revue,
//! les gardes, l'exécution sur une `Lecture` en lecture seule) : la base
//! n'arrête pas un défaut qui contournerait le mode de session.
//!
//! # L'authentification d'une clé MCP — Story 32-5b
//!
//! `cles_mcp::authentifier(&Lecture, &Trace, jeton)` est le **seul**
//! constructeur de production d'un `ContexteAcces` du canal MCP : il cherche la
//! clé sur la `Lecture` et écrit sa dernière utilisation sur la `Trace`, si
//! bien qu'il fonctionne sur une `Lecture` en `READ ONLY`. Le middleware de
//! `/api/v1` ne le connaît pas (une clé MCP y échoue en 401) ; la garde de
//! l'endpoint `/mcp` l'appelle (`mcp::transport`, Story 32-6a).
//!
//! # Le serveur MCP — Story 32-6a
//!
//! `serveur_mcp::reglages_actifs` (la règle « servi », lue à chaque requête
//! vers `/mcp` avant l'authentification) **déroge** aux règles R1, R2 et R7 :
//! aucune portée ne s'y applique, d'où ni `ContexteAcces`, ni `exiger`, ni
//! test de refus. `serveur_mcp::etat` (l'outil `kesh_version`) suit le motif.

#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod acces;
pub mod carnet;
pub mod cles_mcp;
pub mod ecritures;
pub mod envois_mcp;
pub mod exercices;
pub mod factures;
pub mod factures_fournisseur;
pub mod pieces;
pub mod plan_comptable;
pub mod poignees;
pub mod postes_ouverts;
pub mod prompts_mcp;
pub mod rapports;
pub mod serveur_mcp;
