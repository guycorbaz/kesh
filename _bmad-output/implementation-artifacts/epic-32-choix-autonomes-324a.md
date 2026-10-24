# Epic 32 — registre des choix autonomes de la story 32-4a (couche de services : écritures, postes ouverts, contacts)

À verser par **union** dans `epic-32-choix-autonomes.md` au merge (`docs/agents-orchestration.md` § 1). Fiche :
`implementation-artifacts/32-4a-services-ecritures-contacts.md`. Date : 2026-10-24. Agent de fiche : Opus 5.5, en
autonomie. Relevé au code de `89d68704` (32-3 fusionnée).

---

### C-32-4a-1 — Cinq lectures migrées : celles que serviront les outils de la 32-9a et de la 32-9b

- **Contexte** : l'epic (§ 32-4) nomme `routes/journal_entries` (liste, détail), les postes ouverts et
  `routes/contacts` (liste, détail) ; C-32-0-4 borne la couche de services aux **lectures que serviront les outils**.
  Les trois fichiers portent **sept** handlers de lecture (relevé de la fiche, fait 1) : les cinq nommés, plus
  `get_lettering` (`GET /letterings/{key}`) et `get_lettering_proposals` (`GET /accounts/{id}/lettering-proposals`),
  qu'aucun outil prévu ne sert (32-9a : « postes ouverts » ; 32-9b : « contacts (liste, détail) »).
- **Retenu** : migrer les **cinq** ; laisser `get_lettering` et `get_lettering_proposals` dans leurs handlers, **nommés**
  à l'inventaire des sites non résolus et figés par la garde (AC12 (iii)) — un handler de lecture ajouté plus tard sur
  `state.pool` dans ces fichiers rougit, et se trie.
- **Écartées** : migrer les sept (« aucun handler de lecture des modules visés », lecture littérale de l'AC de l'epic) —
  deux services sans consommateur, contraire à C-32-0-4 et sans gain pour la 0.16.0 ; ne rien figer pour les deux
  restants — une liste qu'on a pensé à traiter, non un inventaire clos (`CLAUDE.md`, *Inventorier les sites NON
  RÉSOLUS*).
- **Réversible** : oui ; la story qui ajoutera un outil servant ces lectures les migrera — **aucune échéance planifiée**
  (ni la 32-9a, ni la 32-9b, ni la 0.17.0, dont les « propositions » sont celles de l'agent, `epic-32-serveur-mcp.md:69`,
  non les paires de lettrage ; validation P1, F6). *(Écart à la lettre de l'AC de l'epic tranché ici seul : soumis à
  Guy, fiche § *Points à soumettre à Guy*, Q1 ; l'epic est annoté — validation P1, R7, 2026-10-24.)*

### C-32-4a-2 — `contact_persons::list_persons` hors périmètre

- **Contexte** : `GET /contacts/{id}/persons` (`lib.rs:877-880`) lit les personnes d'un contact, dans un **autre**
  module (`routes/contact_persons.rs`) ; la 32-9b ne nomme que « contacts (liste, détail) ».
- **Retenu** : hors périmètre, nommé au *Hors périmètre* ; si la 32-9b sert les personnes, elle migrera cette lecture.
- **Écartée** : l'ajouter (un cinquième module, à la limite du découpage préventif, sans consommateur décidé).
- **Réversible** : oui.

### C-32-4a-3 — Noms des modules de services : `ecritures`, `postes_ouverts`, `carnet`

- **Contexte** : la 32-3 nomme en français (`rapports`, `plan_comptable`). Un module `services::contacts` heurterait, dans
  `routes/contacts.rs`, l'import `kesh_db::repositories::contacts` que les handlers mutants gardent.
- **Retenu** : `services/ecritures.rs`, `services/postes_ouverts.rs`, `services/carnet.rs` (le « carnet d'adresses » du
  manuel utilisateur).
- **Écartée** : `services/contacts.rs` avec un alias à l'import — un nom qui se lit deux fois pour deux choses.
- **Réversible** : oui (renommage interne).

### C-32-4a-4 — `Portee::ContactsRead` ajoutée maintenant, sans persistance

- **Contexte** : le service des contacts exige la portée `contacts:read` (C-32-0-7) ; la 32-3 n'a posé que `BooksRead`
  et `InvoicesRead` (C-32-3-4 : « la 32-5 ajoute les autres et leur persistance »). Sans la variante, l'exigence MCP du
  carnet ne s'écrit pas — ou s'écrirait `None` (jamais servi), ce qui mentirait sur la 32-9b.
- **Retenu** : la variante `ContactsRead` seule, dans `services/acces.rs`, doc-comment compris ; ni chaîne `contacts:read`,
  ni persistance, ni analyse — c'est la 32-5. Conséquences sur `tests/services_motif.rs`, écrites à l'AC8 : le `match`
  exhaustif d'`autre_portee` (l. 894-899) gagne un bras, et `TOUTES` (l. 869) passe à trois portées pour rester vrai.
- **Écartées** : `portee_mcp: None` pour le carnet (faux) ; `BooksRead` pour le carnet (les contacts sont des données
  personnelles : portée propre, décision de l'epic).
- **Réversible** : oui.

### C-32-4a-5 — Défauts, bornes et validation de la demande dans le service ; une constante, écrite une fois

- **Contexte** : le risque nommé par l'epic est la borne de pagination perdue au déplacement. Le MCP appellera le
  service sans passer par le DTO d'Axum : une borne laissée au handler ne le protégerait pas. Deux des trois dépôts
  n'écrêtent rien (`contacts::list_by_company_paginated`, `letterings::open_items`) : la borne du handler est
  aujourd'hui la **seule**.
- **Retenu** : chaque demande porte `limit: Option<i64>` et `offset: Option<i64>` ; le service applique le défaut, puis
  `clamp(1, MAX)` et `max(0)`, avec les valeurs d'aujourd'hui (écritures 50 / 500 ; postes ouverts 50 / 500 ; contacts
  20 / 100). Les constantes passent au service (`pub`), et les fonctions `default_*` des DTO serde s'y réfèrent ; le
  plafond commun des propositions de lettrage (`MAX_LIMIT`, `letterings.rs:292`, partagé aujourd'hui avec les postes
  ouverts) se lit du service. Les analyses et contrôles de la demande (dates, montants, `accountId`, `asOf`) passent au
  service dans l'ordre d'aujourd'hui, messages identiques — **y compris leurs noms de paramètres en camelCase**
  (`dateFrom invalide (…)`) : le vocabulaire de l'API, que l'outil MCP de la 32-9a reprendra ou traduira.
- **Écartées** : bornes au handler (le MCP les contournerait) ; messages reformulés (changement visible de l'API).
- **Réversible** : oui.

### C-32-4a-6 — L'ordre des lectures d'aujourd'hui est conservé, l'écart de la liste des écritures compris

- **Contexte** : `get_contact` lit le contact **avant** la société (`contacts.rs:613`, puis `:618`) ; `list_contacts` lit
  la société d'abord (`:570`) ; `list_journal_entries` ne lit **aucune** société (`journal_entries.rs:456`, sur
  `current_user.company_id`) — alors que `docs/MULTI-TENANT-SCOPING-PATTERNS.md` écrit « Every handler validates company
  exists ».
- **Retenu** : ordres conservés tels quels (comportement inchangé : un jeton orphelin rend aujourd'hui une liste
  d'écritures vide, non un 500) ; la note de la 32-3 au document des motifs (l. 273-276) nomme les cinq lectures et dit
  l'exception de la liste des écritures, telle qu'elle est.
- **Écartées** : ajouter le contrôle de société à la liste (changement de comportement, hors d'un rollout mécanique) ;
  le taire (le document dirait faux sur un site que la story touche).
- **Réversible** : oui ; si l'on veut le contrôle, une issue le demandera.

### C-32-4a-7 — La garde de transaction de `get_journal_entry` suit le code dans le service

- **Contexte** : `get_journal_entry_reads_in_one_transaction` (`journal_entries.rs:1005-1080`, Story 15-1b-0) lit le
  **corps du handler** et exige `pool.begin()` une fois, aucun `acquire()`, les quatre lecteurs sur `tx`, et
  `find_by_id(&state.pool` hors transaction. Le corps quitte le handler : la garde rougirait (« get_journal_entry
  introuvable » ne tombe pas — le handler reste —, mais `pool.begin()` y compterait 0).
- **Retenu** : la garde **se déplace** au `#[cfg(test)] mod tests` de `services/ecritures.rs`, même nom, mêmes
  assertions, sur le corps de `pub async fn detail(` ; **trois** jetons changent (validation P1, R3 = F3 — « deux » à
  la rédaction) : le compte de la transaction, `pool.begin()` → `pool().begin()` (la première ne figure pas dans
  `lecture.pool().begin()` et compterait zéro), compté sur le corps sans blancs ; `state.pool` → `lecture.pool()` dans
  l'écart assumé (`find_by_id(lecture.pool()`) ; `state.pool` → `.pool()` dans le refus d'un lecteur sur le pool. Elle disparaît de
  `routes/journal_entries.rs` ; le diff des deux versions se recopie au Dev Agent Record. Mutation M13.
- **Écartées** : la laisser dans `journal_entries.rs`, pointée sur `../services/ecritures.rs` (une garde loin de ce
  qu'elle garde) ; la supprimer (le compilateur ne voit pas une connexion acquise à côté de la transaction, motif
  d'origine de la garde).
- **Réversible** : oui.

### C-32-4a-8 — La présentation des contacts reste au handler ; le service rend la société

- **Contexte** : `contact_response_with_label` (`contacts.rs:510-522`) calcule le libellé des conditions de paiement
  avec `state.i18n` et la langue tirée du contact et de la société. `I18nBundle` vit dans `AppState` (R1 l'interdit au
  service).
- **Retenu** : `carnet::lister` rend `PageContacts { page, societe }`, `carnet::detail` rend `FicheContact { contact,
  societe }` ; le libellé reste au handler (présentation, R6).
- **Écartée** : passer `&I18nBundle` au service (le libellé n'est pas une lecture des livres ; le MCP aura sa propre
  mise en forme).
- **Réversible** : oui.

### C-32-4a-9 — Un fichier de tests neuf pour le rollout ; la `Lecture` en lecture seule partagée

- **Contexte** : `tests/services_motif.rs` (2177 lignes) porte le motif et sa garde ; les tests de refus et de lecture
  seule des cinq services sont neufs. `pool_lecture_seule` et `lecture_seule` (`services_motif.rs:833-855`) seraient
  recopiées (DRY).
- **Retenu** : `tests/services_rollout.rs` neuf (la 32-4b y ajoutera les siens) ; `pool_lecture_seule` et
  `lecture_seule` déplacées dans `tests/common/lecture_seule.rs` (`pub mod lecture_seule;` de `tests/common/mod.rs`,
  sous son `#![allow(dead_code)]`), employées par les deux fichiers — `services_motif.rs` gagnant `mod common;`, qu'il
  ne déclare pas aujourd'hui (validation P1, R5 = F4) ; la **garde lexicale** reste dans `services_motif.rs` et s'y
  étend (R8). Le montage HTTP (`spawn_app`, `forge_jwt`) et les constructeurs de contexte se **recopient** dans
  `services_rollout.rs` : convention du dépôt (`spawn_app` dans 68 binaires de test, `forge_jwt` dans 29), et la 32-4b
  ajoute au même fichier, sans troisième copie (validation P1, F4). **Ordre avec la 32-4b** (validation P1, F8) : elle
  part de `main` après la fusion de la 32-4a ; à défaut, celle qui fusionne en second rebase, ajoute à
  `services_rollout.rs` sans le recréer et porte le compte des fichiers de `services/` à la somme.
- **Écartées** : tout dans `services_motif.rs` (fichier déjà long, et un binaire qui mêle motif et rollout) ; recopier
  l'aide.
- **Réversible** : oui.

### C-32-4a-10 — Points de #738 repris : A5, P2-2, et B4 pour sa forme coupée ; les autres y restent

- **Contexte** : #738 trace onze LOW de la revue de la 32-3 et deux de sa P2. La 32-4a étend la garde (ii) et touche
  `services_motif.rs`.
- **Retenu** : **A5** (garde (i)/(ii) sans témoin positif, (ii) jamais rougie par une mutation) — la garde étendue gagne
  son témoin et ses mutations (M15) ; **P2-2** (aucune ligne de `TABLE_DES_TRACES` ne trace les journaux **sans**
  filtre) — **une dix-neuvième ligne** `journaux_affichage_sans_filtre` (mutation M17), la ligne 7 **restant filtrée**
  (mutation M17′). *(Révisé à la validation P1, R1 : la rédaction passait la ligne 7 à `journal: None` ; or elle est
  la seule qui exerce « affichage avec filtre » — le bras d'affichage de `tracer` (`rapports.rs:378-390`) appelle
  `emit_report_audit`, la ligne 8 passant par `emit_report_export_audit` — et la mutation « l'affichage trace `None` »
  aurait survécu. Écartée aussi : garder dix-huit lignes et déclarer la perte.)* **B4**, repris pour sa seule forme
  « `.pool` coupé sur deux lignes », sur les gardes (ii) et (iii) que la story écrit (validation P1, R2 = F2 : la forme
  est dans `letterings.rs:243-244`, `:544-545`, `:566-567`) — détecteur sur le corps aplati, témoin coupé, M16 ; ses
  autres formes et la garde (i) restent. Restent sur #738, avec la raison au *Hors périmètre* : B1, B2, B3 (conception
  du `Rapport`, que la 32-4a ne produit pas), B4 hors la forme coupée (aucun faux vert aujourd'hui), E3, E4 (rapports),
  B5 (doc-comment de `routes/reports.rs`, un module de plus), A3, A4, P2-1 (texte de la fiche 32-3, historique), A6
  (non vérifiable).
- **Réversible** : oui.

### C-32-4a-11 — Les bornes du service des écritures : mutations équivalentes, déclarées

- **Contexte** : `journal_entries::list_by_company_paginated` écrête **aussi** (`kesh-db/src/repositories/journal_entries.rs:935-936`,
  `MAX_LIMIT` = 500 l. 777) et rend les valeurs écrêtées : retirer l'écrêtage du service ne change rien d'observable.
- **Retenu** : M8 déclarée **équivalente**, la ligne du dépôt citée au Dev Agent Record ; le test de bornes des écritures
  reste (il tient le contrat que le MCP recevra, et rougirait si le dépôt perdait le sien) ; le défaut (`None` → 50),
  que le dépôt ne connaît pas, est tenu par le test et tué par M8′.
- **Écartée** : modifier le dépôt pour rendre la mutation tuable (hors périmètre, `kesh-db`).
- **Réversible** : oui.

### C-32-4a-12 — Ni CHANGELOG, ni manuel, ni `docs/api-external.md` : rien de visible ne change

- **Contexte** : comme la 32-3 (C-32-3-41), aucun statut, corps ni borne ne change.
- **Retenu** : contrôle exigé (AC16) — `docs/api-external.md` l. 213, 216-218, 405, 441, et 426-433 (l'ordre des refus
  des postes ouverts ; validation P2, R6), les manuels et leurs PDF —,
  aucune modification ; une phrase au document des motifs (C-32-4a-6) et le doc-comment de `services/mod.rs`.
- **Réversible** : oui.

---

## Choix du développement (2026-10-24, Opus 5.5, en autonomie)

### C-32-4a-13 — Le compte des `async fn` toutes visibilités rétabli, à côté du compte au niveau du fichier (P3-3)

- **Contexte** : la validation P3 (P3-3) relève que le compte au niveau du fichier de l'AC12 (iii), qui a remplacé en P2
  le compte des `async fn` toutes visibilités, ne le couvre pas : une `async fn` non `pub` qui lirait le pool sous un
  autre nom que `state` (`s.pool`) n'ouvre aucun corps et échappe au détecteur ; le compte des `async fn` la voyait.
- **Retenu** : les deux comptes, dans `ac12_iii_inventaire_des_sites_non_resolus` : `pub async fn` (7 / 5 / 5, par
  `corps_des_handlers`), `async fn` toutes visibilités (7 / 5 / 5, motif `^\s*(pub(\([^)]*\))?\s+)?async\s+fn\b` sur les
  lignes de code), et l'égalité « occurrences du fichier = somme des corps ». M16′ rougit par les deux derniers.
- **Écartée** : écrire la limite seulement (aux limites de l'AC12) — un compte d'une ligne la ferme pour un coût nul.
- **Réversible** : oui.

### C-32-4a-14 — La définition du corps d'un handler : celle de `corps_des_handlers` (P3-8)

- **Contexte** : la validation P2 (R3) a retenu, pour (ii) et (iii), une seule définition du corps — celle de
  `corps_des_handlers` (de la ligne `pub async fn <nom>` à la première ligne égale à `}`) — contre la définition
  inverse que R3 préférait (« jusqu'au `pub async fn` suivant »). Le registre ne portait pas ce choix.
- **Retenu** : la définition de `corps_des_handlers`, choisie par **nom exact** (`corps_du_handler`) ; la (ii) de
  `list_accounts` s'y ramène (entre deux handlers, `accounts.rs` ne porte que des doc-comments). Ce qu'elle ne voit pas
  — une aide hors de tout corps — est vu par le compte au niveau du fichier (C-32-4a-13). La garde de transaction
  déplacée (AC4) garde, elle, sa propre délimitation, déplacée telle quelle.
- **Écartée** : « jusqu'au `pub async fn` suivant » — englobe les aides posées entre deux handlers dans le corps du
  premier, et attribue leur `state.pool` au mauvais handler.
- **Réversible** : oui.

### C-32-4a-15 — La conversion `CoreJournal → DbJournal` se fait au service

- **Contexte** : AC3 laisse le choix (service ou handler).
- **Retenu** : `DemandeListeEcritures.journal` porte le type de `kesh-core` (`kesh_core::accounting::Journal`), celui du
  DTO ; `ecritures::lister` le convertit (`DbJournal::from`), comme le faisait le handler. Une valeur invalide reste un
  refus de l'extracteur `Query`, avant le service ; l'outil MCP de la 32-9a analysera vers le même type.
- **Écartée** : la demande en `kesh_db::entities::Journal` — le vocabulaire de l'API n'est pas celui du dépôt.
- **Réversible** : oui.

### C-32-4a-16 — Les DTO gardent leurs défauts serde ; le handler passe `Some(limit)`

- **Contexte** : `ListJournalEntriesQuery` et `OpenItemsQuery` portent `limit`/`offset` en `i64` avec défauts serde ;
  `ListContactsQuery` en `Option<i64>` sans défaut.
- **Retenu** : les DTO ne changent pas de forme ; `default_limit` et `default_open_items_limit` rendent
  `ecritures::LIMITE_DEFAUT` et `postes_ouverts::LIMITE_DEFAUT` ; le handler passe `Some(params.limit)` /
  `Some(params.offset)` (contacts : l'`Option` telle quelle). Le défaut du service sert au MCP (`None`) ; celui du DTO,
  qui est la même constante, sert à la route. Une seule valeur, deux chemins.
- **Écartée** : passer les DTO en `Option<i64>` (forme de désérialisation changée — `limit=` vide, par exemple — hors
  d'un rollout mécanique).
- **Réversible** : oui.

### C-32-4a-17 — La garde déplacée garde son nom ; T2 à T4 en un commit

- **Contexte** : AC4 laisse le nom au choix ; T2, T3, T4 sont trois tâches.
- **Retenu** : `get_journal_entry_reads_in_one_transaction`, dans `services/ecritures.rs` (un `grep` du nom retrouve la
  garde citée par la 15-1b-0 et les commentaires du handler). T2 à T4 en **un** commit : les tests de T1 (ii) importent
  les trois services, et un commit par tâche aurait laissé deux commits où `cargo clippy --all-targets` ne compile pas.
- **Réversible** : oui.

### C-32-4a-18 — Le bras `ContactsRead` d'`autre_portee` (`services_motif.rs`) est un `panic!`

- **Contexte** : `autre_portee` (`match` exhaustif sur `portee_exigee(svc)`) gagne un bras (AC8 (1)) ; aucun service
  de rapport n'exige `ContactsRead`.
- **Retenu** : `Portee::ContactsRead => panic!("aucun service de rapport n'exige ContactsRead")` — inatteignable par
  `portee_exigee`, et bruyant si un rapport l'exigeait un jour.
- **Écartée** : rendre une portée arbitraire (le test mentirait sur « l'autre portée »).
- **Réversible** : oui.

### C-32-4a-19 — Le chemin de succès des filtres se teste par la route, sur des fixtures propres au test

*(Revue de code P1, B1 = E1 = E2, A4.)*

- **Contexte** : aucun test n'exerçait la recopie des filtres (`ecritures::lister` → `JournalEntryListQuery`, handler
  des contacts → `DemandeListeContacts` → `ContactListQuery`) ni les défauts du tri du carnet.
- **Retenu** : deux tests **par la route** dans `tests/services_rollout.rs` — `filtres_des_ecritures_par_la_route`
  (douze lignes : chacun des sept filtres seul, l'égalité et l'inclusion des deux bornes) et
  `tri_par_defaut_et_filtres_du_carnet` (onze lignes : tri par défaut, trois colonnes, deux sens, cinq filtres), qui
  tiennent ensemble le handler, le service et le dépôt ; le tri par défaut est **aussi** vérifié au service, demande
  vide, sous un contexte MCP (le chemin de la 32-9b). Fixtures synthétiques **posées dans chaque test** (trois
  écritures, quatre contacts aux dates de création et de modification explicites), et non ajoutées à `poser_livres`,
  dont les autres tests comptent les lignes (`total >= 1`, une écriture, un contact). Les ensembles d'écritures sont
  comparés sans ordre (le tri des écritures n'est pas l'objet) ; les contacts dans l'ordre rendu.
- **Écartée** : des tests au service seulement — ils ne voient pas la recopie du handler, que B1 nomme.
- **Réversible** : oui.

### C-32-4a-20 — La campagne de la remédiation ne relinke que `services_rollout`, et échantillonne les recopies du handler

- **Contexte** : la consigne demande de borner les écritures disque ; le pilote de la T6 lançait
  `cargo nextest run --workspace -E 'binary(…)'`, qui reconstruit tous les binaires de test de `kesh-api` à chaque
  mutation de `src/`.
- **Retenu** : pilote `kesh-gate-logs/324a-p1r-mutations-pilote.py`, même protocole (motif vérifié unique, diff
  sauvegardé, `git checkout -- <f> && touch <f>`, résidu nul), mais `cargo nextest run -p kesh-api --test
  services_rollout` ; dix-neuf mutations, M19 à M37. Côté handler, **quatre** des sept recopies des contacts
  (`search`, `is_supplier`, `include_archived`, `sort_direction`) et **une** des onze des écritures (`journal`) sont
  jouées ; les autres ont chacune leur ligne dans la table des tests, **sans** être jouées — non revendiquées comme
  tuées.
- **Écartée** : jouer les dix-huit recopies du handler (treize compilations de plus pour des lignes que la table
  désigne déjà une à une).
- **Réversible** : oui (le pilote se relance avec d'autres identifiants).

### C-32-4a-21 — AC14 : les diffs restent aux journaux, une ligne par mutation au Dev Agent Record

*(Revue de code P1, A2.)*

- **Contexte** : l'AC14 demande les assertions « recopiées au Dev Agent Record avec leur diff » ; le Record renvoie à
  `kesh-gate-logs/324a-mut-<id>.diff`.
- **Retenu** : les diffs complets restent **hors dépôt** (consigne : rapports et journaux hors dépôt) ; le pilote, qui
  porte en clair chaque couple (ancien texte, nouveau texte), y est aussi. Pour M19 à M37, le Dev Agent Record recopie
  **en une ligne** le remplacement de chaque mutation et la ligne de table qui rougit ; pour M1a à M18, la table de la
  T6, qui décrit chaque mutation en une phrase, n'est pas réécrite (relevé daté).
- **Écartée** : recopier les trente-cinq diffs de la T6 dans la fiche (une centaine de lignes sans lecteur, et une
  entrée de passe passée réécrite).
- **Réversible** : oui.

### C-32-4a-22 — A1 : le doc-comment de `services/mod.rs` nomme les quatre branches non atteintes

- **Contexte** : le doc-comment en nommait deux, le Dev Agent Record quatre.
- **Retenu** : aligner le doc-comment sur le Record — compte non lettrable (refusé par `require_viewable_account`
  avant toute lecture des postes, `letterings/open_items.rs:355`) et contact à conditions de paiement (libellé mis en
  forme au handler) ajoutés, avec la raison de leur place.
- **Écartée** : dire au Record que ces deux-là sont hors inventaire — vrai, mais le doc-comment est le texte que lit
  le développeur de la 32-6.
- **Réversible** : oui.

### C-32-4a-23 — B3, E5 et E4 ne se corrigent pas dans la 32-4a : ils se tracent

- **Contexte** : B3 (commentaire de `kesh-db/src/repositories/journal_entries.rs:934`, « la source de vérité est le
  route handler »), E5 (le plafond 500 écrit deux fois, `services::ecritures::LIMITE_MAX` et `MAX_LIMIT` du dépôt),
  E4 (`analyser_date` sans la plage 1000-9999).
- **Retenu** : rien dans le code — `kesh-db` reste non touché (AC15), et E4 changerait un comportement que la story
  déplace sans le changer (déjà #742). Le texte des commentaires à verser sur **#738** (B3, E5) et **#742** (E4) est
  demandé à l'orchestrateur, qui seul peut écrire sur GitHub.
- **Réversible** : oui.
