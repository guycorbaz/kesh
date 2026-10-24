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
//!   `tests/services_motif.rs` le vérifie à l'exécution, la 32-6 l'imposera par
//!   la base pour la `Lecture`. Ce qu'un handler doit lire pour sa présentation
//!   (le contexte PDF) passe par une fonction de service. L'action d'audit reste
//!   un **littéral** à chaque `NewAuditLogEntry::user(` (garde
//!   `tests/audit_label_registry.rs`).
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
//!    service, inchangée ; la clé MCP de la 32-5 écrira sa dernière utilisation
//!    sur la `Trace`.
//! 3. Aucune autre : les générateurs de `kesh-report`, `ReportPeriod::resolve`,
//!    `resolve_scope`, `rapports::identite_societe`, `helpers::company_for_ids`,
//!    `accounts::list_by_company` et `letterings::letterable_account_ids` ne
//!    font que lire — établi par la lecture du code, et confirmé, sur les
//!    branches que ses fixtures exercent, par `tests/services_motif.rs`, qui les
//!    exécute sur une connexion en lecture seule.
//!
//! # Ce qui reste à la 32-6
//!
//! La fabrication d'une `Lecture` en lecture seule — mode de session,
//! utilisateur MariaDB limité au `SELECT`, ou les deux, et le lieu où elle se
//! fabrique — est décidée par Guy (Q11) et réalisée par la 32-6. En 32-3, les
//! deux poignées enveloppent le même pool de production ; seule leur
//! **distinction** est posée ici. La garantie réelle contre une écriture sur la
//! `Lecture` sera celle de la base, en 32-6 ; d'ici là, c'est la revue de code.

#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod acces;
pub mod plan_comptable;
pub mod poignees;
pub mod rapports;
