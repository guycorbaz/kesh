# Story 32.11 : brancher un agent, de bout en bout

Status: ready-for-dev

*Fiche rédigée le 2026-10-27, en autonomie (Opus 5.5), worktree `kesh-32-11`, branche `story/32-11` partie de
`origin/main` `ed981c7d` (32-1 à 32-6c et 32-8 fusionnées ; 32-7, 32-9a, 32-9b, 32-10a et 32-10b non fusionnées, leurs
fiches lues en lecture seule dans leurs worktrees). Registre : `epic-32-choix-autonomes-3211.md` (C-32-11-1 à
C-32-11-29). Clé courte : `3211`. Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs` seulement. Les
numéros de ligne cités sont ceux de `ed981c7d`, sauf mention d'une fiche voisine (lue à la date de rédaction).*

⛔ **Développement APRÈS la fusion de toutes les autres stories de l'Epic 32** : 32-7, 32-9a, 32-9b, 32-10a, 32-10b. La
32-11 décrit et éprouve ce que **leur** code fait ; l'étape 0 vérifie que `git log --oneline origin/main` porte chacune,
et, s'il en manque une, **le développement ne commence pas** (l'agent le dit à l'orchestrateur). Aucun AC n'a de variante
« si la story n'a pas fusionné ». **Tout ce que cette fiche dit du code de ces stories se recontrôle à l'étape 0**
(§ *À recontrôler*), ligne lue citée au Dev Agent Record ; une phrase prescrite ici que le code ne confirme pas se
réécrit selon le code.

⚠️ **Documentation exécutée** (précédent : la 31-4, 8 + 4 passes, 15 des 24 MEDIUM de revue de l'Epic 31). Pour éviter
son recyclage, cette fiche **fixe trois clôtures dès le départ** : le **parcours** est une liste close de gestes (AC-A2) ;
**un défaut que le parcours révèle ne se corrige pas ici** (AC-A7) ; **aucune garde de texte nouvelle** hors du seul
compte des routes (AC-T1, retrait pré-déclaré). La justesse du manuel est portée par le **journal d'exécution**, le
**rejeu depuis le PDF** et la relecture en revue — non par des gardes.

## Story

En tant que Guy, qui installera Kesh 0.16.0 à neuf et y branchera Claude Code pour qu'un agent lise ses livres, je veux
un parcours **joué réellement** — activer le serveur, créer une clé, appeler un outil, valider l'envoi, le relire,
relire la trace, restaurer une sauvegarde —, joué contre le serveur par l'agent de développement en client HTTP, décrit
pas à pas au manuel et rejoué depuis le PDF, puis **éprouvé avec Claude Code** à ma recette, un **test de
contrat** qui rougit en CI si une réponse de `/mcp` s'écarte de la révision figée du protocole, une **description
détaillée du fonctionnement de MCP** dans les deux manuels, et des textes publics (`docs/api-external.md`, brochure,
`README.md`, feuille de route du site) qui disent vrai, afin de savoir ce que l'agent peut lire, ce qu'on me montre avant
qu'une donnée parte, et ce que Kesh ne garantit pas.

## Origine

- **Epic 32, § 32-11** (`planning-artifacts/epic-32-serveur-mcp.md:516-541`) : le parcours entier joué sur une
  démonstration avec un client réel ; le test de contrat en CI (requêtes et réponses de la révision figée, rejouées dans
  un test d'intégration Rust, sans réseau) ; `docs/api-external.md` § 6.4 et § 9 ; manuel d'administration (*Brancher un
  agent*), manuel utilisateur, brochure ; une restauration jouée (clés, réglages, prompts). Trois modules, risque moyen ;
  « la fiche fixe le périmètre du parcours dès le départ ».
- **Décision de Guy du 2026-10-25** (fiche 32-6c, point 5) : « ajoute la description détaillée du fonctionnement de
  MCP » — aux **deux** manuels (epic, l. 531-536).
- **Report de l'orchestrateur du 2026-10-25** (epic, l. 539-541 ; fiches 32-6 R13, 32-10a C-32-10a-29) : `README.md:233`
  et `website/roadmap.html:375` deviennent faux (« montrées avant de partir », faux pour une clé `dispensée` ; « aucune
  écriture par l'agent », faux depuis `proposer_prompt`).
- **Décision de Guy du 2026-10-25 au soir** (epic, l. 36) : **Hermes est hors du périmètre** ; seul Claude Code est
  nommé comme client.
- **Spike 32-2** (`planning-artifacts/epic-32-spike-mcp.md`) : § 1.3 (révision `2025-11-25`, sans session), § 2.1
  (Claude Code), § 2 (« non éprouvé tant que Guy ou la 32-11 n'a pas rejoué le § 2.4 », l. 206-207), § 2.4 (gestes et
  observations (a) à (h), l. 306-351), § 6 entrée 32-11 (l. 973-975 : séquence du contrat, parcours joué, observations consignées).
- **Reports des fiches voisines** — 32-6 (`32-6-endpoint-mcp.md:1303-1323` : B4, A3, `api-external.md`), 32-7
  (C-32-7-13, C-32-7-22 : `sec:mcp-client` lui appartient, la 32-11 l'amende en place ; demande 11 : l'épreuve des
  mandataires), 32-8 (`32-8-validation-avant-envoi.md:1441-1443` : observation (d), le contrat gagne le dépôt et la
  relecture ; C-32-8-26 et C-32-8-27), 32-9 (`32-9-outils-de-lecture.md:1330-1333`, `:1347-1349`, `:1378` — demande 2, relue à `c5a063da` du worktree `kesh-32-9` : Q2 des
  pièges, un `tools/call` d'outil de données au contrat), 32-10a (`32-10a-prompts-en-base.md:1388-1394` : invoquer un
  prompt dans le client), 32-10b (AC-19 et AC-20 : le compte des routes, puis la 32-11 au dernier état).

## Ce que le code et les textes disent (relevé le 2026-10-27, `ed981c7d`)

1. **Le serveur MCP** : `POST /mcp`, Streamable HTTP sans session, JSON-RPC 2.0, révision `2025-11-25`, `2025-06-18`
   acceptée, réponses JSON (`crates/kesh-api/src/mcp/mod.rs:1-9`) ; l'ordre des contrôles en douze étapes, du pair TCP à
   `reponse::rendre` (`mod.rs:11-34`) ; `VERSIONS_SERVIES` (`:95`), corps borné à 1 Mio (`MCP_CORPS_MAX`, `:88`), délai
   d'outil 30 s (`DELAI_OUTIL`, `:92`), trois connexions au plus pour les outils (`MCP_POOL_CONNEXIONS`, `:85`). C'est
   la charpente du « parcours d'une requête » de la description détaillée (AC-C1).
2. **Les outils** : à `ed981c7d`, `kesh_version` seul (`mcp/outils/mod.rs:1-5` ; `mcp/registre.rs:42`, la doc de
   `standard()`, et `:46`, son corps `Self::vide().avec(Box::new(KeshVersion))`) et le nom
   réservé `kesh_lire_envoi` (`registre.rs:22`). La 32-9a en ajoute quinze, la 32-9b les contacts et les pièces
   (fiche 32-9 : `kesh_plan_comptable`, `kesh_ecritures`, `kesh_ecriture`, `kesh_grand_livre`, `kesh_exercices`,
   `kesh_bilan`, `kesh_compte_de_resultat`, `kesh_postes_ouverts`, `kesh_factures`, `kesh_facture`, `kesh_echeancier`,
   `kesh_balance`, `kesh_balance_agee`, `kesh_factures_fournisseur`, `kesh_facture_fournisseur`, `kesh_contacts`, `kesh_contact`,
   `kesh_pieces`, … — **liste à relire au registre à l'étape 0**, non à recopier d'ici).
3. **Les tests MCP existants** sont rangés par garde, non par clause du protocole : `tests/mcp_transport_e2e.rs` (2081
   lignes, dont `initialize_ping_prompts` l. 1027, `methodes_servies_liste_close` l. 983, `origine` l. 548,
   `enveloppe_http` l. 701, `notification_jamais_traitee` l. 948, `session_refusee` l. 1163), `mcp_reseau_e2e.rs`,
   `mcp_journal_e2e.rs`, `mcp_envois_e2e.rs`, `mcp_admin_routes_e2e.rs`, `mcp_gardes_lexicales.rs`, `mcp_traduction.rs` (`ls
   crates/kesh-api/tests/mcp_*.rs` → sept fichiers). **Aucun fichier ne
   porte le mot « contrat »** (`grep -rn -il contrat crates/kesh-api/tests/mcp_*.rs` → rien). Ils comparent les
   réponses à des constantes de `mcp::textes`, à la route absente, ou à des **littéraux écrits par l'auteur de la
   story** — `initialize_ping_prompts` (l. 1027-1060) écrit le corps d'`initialize` et `INSTRUCTIONS` en toutes lettres,
   négociation `2024-11-05 → 2025-11-25` comprise ; c'est-à-dire à **ce que Kesh a décidé** : aucun ne cite la clause
   de la spécification qui fonde l'attendu (`grep -n -i clause crates/kesh-api/tests/mcp_*.rs` → rien). Le recoupement
   avec K1 à K3 et K6 est donc large ; C-32-11-8 l'assume. Les aides `attendu_depose`, `attendu_en_attente`,
   `attendu_valide`, `attendu_refuse`, `attendu_echu`, `attendu_inconnu` (`tests/common/mcp.rs:626-663`) construisent
   leur attendu par `kesh_api::mcp::textes::texte_envoi_*` (`mcp/textes.rs:119-162`) : un attendu **tiré de Kesh**.
4. **La CI** lance tout binaire de test de `crates/kesh-api/tests/` : `cargo test --workspace -j1 -- --test-threads=1`
   (`.github/workflows/ci.yml:195-199`), MariaDB servi. Un binaire neuf y entre sans réglage.
5. **Les plages admises par défaut** comprennent la boucle locale (`kesh-db/src/entities/mcp_server_settings.rs:26-28` :
   `"127.0.0.0/8"`, `"::1/128"`), filtre activé par défaut (`DEFAULT_ADDRESS_FILTER_ENABLED: bool = true`, `:22`) : un client de la station, ou un mandataire local, est admis sans
   réglage.
6. **Kesh ne journalise pas la révision que le client demande** : `repartiteur.rs:164` la lit
   (`params.get("protocolVersion")`) sans évènement ; seul `transport.rs:121` écrit un `debug!` quand
   `MCP-Protocol-Version` manque. L'observation (h) de la spike (quelle `protocolVersion` Claude Code demande) ne se lit
   donc **pas** au journal de Kesh : il faut voir l'échange — la capture de la recette de Guy (AC-A8, C-32-11-29).
7. **La sauvegarde** emporte les clés, les réglages et les envois du serveur MCP (`kesh-db/src/backup.rs:80-88` ;
   aller-retour colonne par colonne prouvé par `tests/admin_backup_e2e.rs:323`,
   `mcp_tables_roundtrip_column_by_column_and_enabled_server_stays_enabled`) ; la 32-10a y ajoute les prompts et leurs
   versions (sa fiche, AC de restauration). **Aucun essai ne rejoue la restauration par l'écran** sur une installation
   neuve, ni ne rebranche un client après : c'est l'objet de la partie E.
8. **Le compte des routes au manuel est faux de deux** : `admin-manual.tex:2978` écrit « 122 des 128 routes à verbe
   mutant » et « 122 + 4 + 2 = 128 » ; le registre en compte **130**, dont **124** tracées, 4 exemptées, 2 sans effet
   (`tests/audit_route_registry.rs:892-929`, `the_registry_partition_is_what_the_story_declares`). La 32-10a-iii ajoute
   cinq routes tracées et la 32-10b recompte (sa fiche, AC-19) ; **trois stories de l'epic** (32-6a, 32-6b, 32-8b) ont
   laissé ce compte faux — rien ne le relie au registre (C-32-11-10).
9. **Les textes publics** :
   - `docs/api-external.md:181-205` (§ 6.4) : « Il n'existe pas de serveur MCP « Kesh-natif » en v0.2 » (`:185`) et un
     exemple de serveur MCP générique avec une clé `kesh_pat_` (`:187-201`) ; `:831` (§ 9) : « Hors périmètre (non
     planifié pour v0.2) : OAuth/SSO, webhooks, serveur MCP Kesh-natif ». `:35` dit déjà qu'une clé `kesh_mcp_` est
     refusée sur `/api/v1` (32-5) — juste.
   - `admin-manual.tex:2926` renvoie à ce guide pour « MCP » ; `:2963` dit qu'une **clé d'API** donnée au même agent
     lit l'API **sans** la validation avant envoi (32-8c) — juste, et c'est ce que § 6.4 doit dire aussi.
   - `README.md:233` (ligne v0.16.0, 🚧) : « les données **montrées à l'utilisateur avant de partir à l'agent**. Aucune
     écriture par l'agent dans cette version », et, plus haut sur la ligne, « **en lecture seule** » ;
     `website/roadmap.html:375`, mêmes affirmations en anglais (« read-only ») ; `:223` et `:43` disent « a built-in,
     read-only MCP server ». Or `proposer_prompt` (32-10a) **écrit** une version de prompt « à valider », hors de la
     validation avant envoi (fiche 32-10a, l. 203, 821, 837, 1392) : « lecture seule » n'est vrai que **des livres**.
   - `marketing-brochure.tex:480` : « Webhooks et MCP server pour assistants IA conversationnels », sous **Vision
     (v0.4+)** (`:472`) ; la liste des fonctions livrées (`:390-400`) nomme l'API à clés (`:399`), non le serveur MCP.
10. **Le manuel d'administration aujourd'hui** : `sec:mcp-envois` (`:2928`, provisoirement sous *Sécurité*),
    `sec:mcp-journal-filtre` (`:3072`, sous *Monitoring et logs*), une phrase à DSM (`:656`). La section
    `\section{Le serveur MCP}\label{sec:mcp}` et ses sous-sections (activer, clés, filtre, `Origin`, journal, erreurs,
    **`sec:mcp-client`**, et le rangement de `sec:mcp-outils`, `sec:mcp-envois`, `sec:mcp-prompts`) sont **à la 32-7**
    (sa fiche, AC-18) ; `sec:mcp-client` y est écrite « non éprouvée », et renvoie au parcours de la 32-11 (AC-18 (8)).
    **Hermes** : la fiche 32-7 l'écrivait encore à l'AC-18 (8) (l. 846-853) ; la réponse de Guy (l. 1402, l. 1427) le
    retire — à constater à l'étape 0.
11. **Le manuel utilisateur aujourd'hui** ne décrit pas le serveur MCP : il le nomme dans les exclusions de l'export de
    souveraineté (`user-manual.tex:3029-3032`), au journal d'audit (`:3153-3157` : un appel MCP paraît au nom du
    créateur de la clé, reconnaissable à l'action « Outil du serveur MCP appelé ») et à la remise à zéro (`:3208`,
    `:3218`). Ses sections : `\section{Traçabilité (journal d'audit)}` (`:3108`), puis *FAQ* (`:3222`).
12. **#776** (P2, ouverte) : une consultation de rapport par le MCP écrit la même ligne `report.generated` qu'à l'écran,
    au nom du créateur de la clé, sans clé ni canal ; `user-manual.tex:3153-3157` est « incomplet pour les consultations
    de rapport » ; « d'ici là, les deux manuels le disent ».
13. **Les limites connues** (issues ouvertes au 2026-10-27) : #759 (une tâche d'outil abandonnée va au bout, les attentes
    de permis ne sont pas bornées), #761 (LOW de tests de la 32-6b ; son point 7 est le compte du fait 8), #765 (point 5 :
    un contenu échu reste en base **et dans le `.keshbackup`** jusqu'à la purge suivante), #768 (point 1 : une 500 après
    une décision pourtant enregistrée), #770 (point 5 : l'explication de la passerelle Docker derrière DSM, à éprouver sur
    le NAS), #773 (point 8 : la liste des envois bornée à 200, non dite ; point 9 : la purge au seul dépôt).
    **Le manuel en dit déjà une part** : `admin-manual.tex:2949` (« Le contenu d'un envoi échu n'est effacé de la base
    qu'au **dépôt suivant** de la société » — #773 (9)) et `:2238` (section de la sauvegarde : « le contenu d'un envoi
    échu n'est effacé qu'au dépôt suivant de la société, si bien qu'une sauvegarde prise entre-temps le porte encore »
    — #765 (5) entier). #773 (8) et #768 (1) n'y sont pas : `grep -n -F '200' … | grep -i envoi` → aucune phrase sur la borne
    de la liste ; `sed -n 2928,2990p docs/manual/fr/admin-manual.tex | grep -n -i recharg` → la seule ligne rendue (préfixée `14:`, numéro relatif à la fenêtre : 2928 + 14 − 1 = `:2941`) est `:2941`, qui ne vise que le refus d'une décision sur un envoi **changé
    entre-temps** (« la décision est refusée et l'écran propose \emph{Recharger} ») — #768 (1), une 500 **après** une
    décision enregistrée, n'est pas dit.
14. **Claude Code sur la station** : `claude --version` → `2.1.311 (Claude Code)` (la spike lisait `2.1.310`, § 2.1,
    l. 209 ; même sortie à la validation P1, par une commande que le prompt de la lentille interdisait ; la version se relève à
    la recette de Guy, AC-A8 et AC-C2 : l'agent de développement ne lance pas `claude`, pas même `--version`). `socat` est installé (`/usr/bin/socat`). Le dépôt n'a
    **aucune** dépendance de schéma JSON (`grep -n '^name = "jsonschema"' Cargo.lock` → rien). **Rien de ce que la fiche
    dit du comportement de `claude` n'a été exécuté** ; depuis la validation P2, le montage par défaut ne le lance pas
    (C-32-11-28) : ce comportement se constate à la recette de Guy (AC-A8).
15. **Le précédent de la 31-4** (`31-4-repetition-a-blanc.md`) : procédure écrite **en la jouant** (journal
    d'exécution), puis **rejouée depuis le PDF** (T8, l. 749-758) ; garde G22 et fixture scriptée **retirées** au
    recyclage de la validation P3 (C-31-4-15) ; « aucun test ne remplace d'avoir joué les commandes » (l. 617-619).
16. **La démonstration ne porte aucune écriture** (validation P1, R1) : `seed_demo` (`kesh-seed/src/lib.rs:181`) crée la
    société (forme `OrgType::Pme`, `:240`), le plan PME, l'exercice de l'**année civile courante** (« Exercice
    {année} », du 1er janvier au 31 décembre, `:272-284`), les réglages de facturation et les taux de TVA — `grep -nE
    'journal_entries|contacts|invoices' crates/kesh-seed/src/lib.rs` → rien ; même constat aux fiches 32-9 (fait 10) et
    31-4 (l. 61). Un bilan de la démonstration n'a donc **aucun montant non nul**. Le moyen le plus simple d'en saisir
    existe : `POST /api/v1/journal-entries` (`kesh-api/src/lib.rs:486`), **ouvert aux clés d'API `read-write`**
    (`docs/api-external.md:218`, `:265`), corps `{ "entryDate", "journal", "description", "lines": [{ "accountId",
    "debit", "credit" }] }` (`:269`), journaux `Achats`, `Ventes`, `Banque`, `Caisse`, `OD`
    (`kesh-core/src/accounting/balance.rs:25-36`) ; les identifiants se lisent par `GET /api/v1/fiscal-years`
    (`lib.rs:1090`) et `GET /api/v1/accounts` (`lib.rs:913`) ; une clé d'API se crée **à l'écran** seulement
    (`api-external.md:52`, `admin-manual.tex:2899`, `sec:api-keys`). Comptes du plan PME employés par P0 :
    `1020` Banque (`kesh-core/assets/charts/pme.json:6`), `2800` Capital social (`:37`), `3200` Prestations de services
    (`:45`), `6000` Loyers (`:65`).

## Critères d'acceptation

Six groupes : **A** le parcours joué, **B** le test de contrat, **C** les manuels, **D** les textes publics, **E** la
restauration jouée, **T** la garde du compte. Ils se livrent ensemble, sous une seule clé (C-32-11-1).

### A — Le parcours joué

*Montage refondu à la validation P2 (C-32-11-28, C-32-11-29) : l'agent de développement joue le parcours **en client
HTTP**, sans lancer `claude` ; les gestes de Claude Code, et la capture de ses requêtes réelles, sont **la recette de
Guy** (AC-A8). Le pilotage de `claude -p` par l'agent est une variante soumise à Guy (point 1), non retenue par défaut,
esquissée à l'**Annexe**.*

1. **AC-A1 — Le montage.** Une instance du **binaire du worktree** (non l'image publiée, qui n'existe pas avant la
   release — C-32-11-19), sur la recette du § *Montage E2E local* (`CLAUDE.md`) : base `kesh_3211` **vide**, port `P`
   attribué par l'orchestrateur — **deux ports** sont demandés à l'orchestrateur avant le montage, `P` pour cette
   instance et `P_R` pour la seconde (AC-E2) —, `RUST_LOG=info,kesh_api::mcp::reseau=debug` (pour la ligne d'admission
   du filtre, `sec:mcp-journal-filtre`), dossiers neufs `/tmp/kesh-e2e-3211/{inbox,documents,backup}`,
   `KESH_STATIC_DIR` sur le `frontend/build` **du worktree**, construit par `npm --prefix frontend run build` au
   montage (les écrans de P0 à P11 en dépendent). **Sans `KESH_TEST_MODE`** : il n'ajoute que `/api/v1/_test/*`
   (`kesh-api/src/lib.rs:1309-1314`) et un facteur de test (`main.rs:279`), dont aucun geste ne se sert, et le manuel
   décrit une installation sans lui ; les variables `KESH_SMTP_*` factices de la recette restent (aucun geste n'envoie
   de courriel). La démonstration se charge **par l'assistant** (écran), non par SQL ; elle ne porte aucune écriture
   (fait 16), d'où P0. **Le client MCP est `curl`** (C-32-11-28), qui parle à `P` directement : chaque requête du
   parcours est un fichier JSON **écrit avant d'être envoyé**, `kesh-gate-logs/3211/requetes/<id>-<méthode>.json`, et
   chaque réponse est gardée, corps et en-têtes, sous `kesh-gate-logs/3211/reponses/<id>.json` et `<id>.entetes`
   (chemins absolus). Aucun mandataire, aucun `socat`, aucun `claude`. Les navigateurs Playwright de la recette E2E
   servent aux gestes d'écran et aux captures (script jetable sous `kesh-gate-logs/3211/`, jamais versionné) ; si
   l'agent ne peut pas piloter le navigateur, il **s'arrête** (AC-A7, dernier alinéa) — aucune route ne remplace un
   geste d'écran. Versions relevées : `GET /health` (champ `version`), `curl --version | head -1`.
2. **AC-A2 — Le périmètre du parcours est clos** (C-32-11-2). Cette liste est **la seule source** des gestes : il
   compte **ces gestes, et aucun autre** — douze gestes, P0 à P11, **aucun geste conditionnel**. Aucune autre partie
   de la fiche (Dev Notes comprises) n'ouvre de geste ni de variante ; l'Annexe n'est pas une variante du
   développement tant que Guy n'a pas répondu oui au point 1.
   - **P0** — *l'amorçage* (C-32-11-25), avant P1 : (a) l'Administrateur crée à l'écran (`sec:api-keys`) une clé d'API
     `amorcage-3211`, portée `read-write`, dont le secret n'est copié qu'au fichier d'en-tête
     `kesh-gate-logs/3211/amorcage/cle-amorcage.entete` (une ligne `Authorization: Bearer <secret>`, hors dépôt) ; (b) il lit par cette clé l'identifiant de l'exercice de la démonstration
     (`GET /api/v1/fiscal-years`) et ceux des comptes `1020`, `2800`, `3200`, `6000` (`GET /api/v1/accounts`) ; (c) il
     saisit par `POST /api/v1/journal-entries` (fait 16) ces **trois écritures synthétiques**, dates dans l'exercice de
     la démonstration (année `<A>` = celle de l'exercice lu en (b)) :

     | n° | date | journal | libellé | débit | crédit | montant |
     |---|---|---|---|---|---|---|
     | E1 | `<A>-01-15` | `OD` | Apport en capital (amorçage 32-11) | `1020` | `2800` | 17 345.10 |
     | E2 | `<A>-02-10` | `Banque` | Prestation de démonstration | `1020` | `3200` | 4 321.55 |
     | E3 | `<A>-03-05` | `Banque` | Loyer de démonstration | `6000` | `1020` | 1 234.60 |

     Soldes attendus au bilan : `1020` Banque **20 432.05** (17 345.10 + 4 321.55 − 1 234.60), `2800` Capital social
     **17 345.10**, résultat de l'exercice 3 086.95 ; (d) il **révoque** `amorcage-3211` (une clé d'API donnée à un agent
     lit sans validation, `admin-manual.tex:2963`) ; (e) **témoin positif** : il lit le bilan à l'écran *Rapports* et y
     constate 20 432.05 et 17 345.10, capture à l'appui ; il note l'identifiant d'exercice `<X>` pour la requête de P6.
     P0 se joue **depuis cette fiche**, au journal ; il n'entre pas au manuel comme geste (AC-C2). Ses réponses ne vont
     pas sous `reponses/` (elles portent les montants) : `kesh-gate-logs/3211/amorcage/`.
   - **P1** — *Administration → Serveur MCP* : activer le serveur, plages par défaut (fait 5).
   - **P2** — créer la clé `parcours-3211`, portée **`books:read` seule**, validation **requise** (le défaut) ; le secret
     se lit une fois (capture), et n'est copié qu'au fichier d'en-tête de P3.
   - **P3** — *le client HTTP* (C-32-11-28) : le fichier `kesh-gate-logs/3211/cle-parcours.entete` (une ligne,
     `Authorization: Bearer <secret>`, hors dépôt), et les **neuf requêtes du parcours** — R01 à R06, R08 et R09 écrites avant P4, **R07 écrite après R06**, une fois
     `<n>` connu (l'accusé de R06) — sous `requetes/`, chacune envoyée **une fois** par
     `curl -sS -X POST http://127.0.0.1:<P>/mcp -H @<cle-parcours.entete> -H 'Content-Type: application/json'
     -H 'Accept: application/json, text/event-stream' [-H 'MCP-Protocol-Version: 2025-11-25'] --data-binary @<requête>
     -D <id>.entetes -o <id>.json -w '%{http_code}\n'` (en-tête de version sur toute requête qui suit `initialize` ;
     `-H @fichier` à constater par `curl --version`, sinon l'en-tête se passe autrement, sans écrire le secret dans le
     journal) :

     | id | geste | méthode | paramètres |
     |---|---|---|---|
     | R01 | P4 | `initialize` | `protocolVersion: "2025-11-25"`, `clientInfo: {name: "parcours-3211", version: "1"}`, `capabilities: {}` |
     | R02 | P4 | `notifications/initialized` | — (notification, sans `id`) |
     | R03 | P4 | `tools/list` | — |
     | R04 | P5 (c) | `prompts/list` | — |
     | R05 | P5 (c) | `prompts/get` | `name` du standard (relu à l'étape 0) |
     | R06 | P6 | `tools/call` | `kesh_bilan`, `{"fiscalYearId": <X>}` |
     | R07 | P8 | `tools/call` | `kesh_lire_envoi`, `{"envoi": <n>}` (`repartiteur.rs:73-77` à `ed981c7d`, relu à l'étape 0) |
     | R08 | P11 | `initialize` | comme R01, clé révoquée |
     | R09 | P11 | `tools/call` | `kesh_version`, `{}`, clé révoquée |

   - **P4** — les outils : R01 rend `protocolVersion` et aucun en-tête `Mcp-Session-Id` ; R02 rend 202 ; R03 porte
     `kesh_version`, `kesh_lire_envoi` et les seuls outils `books:read` — aucun de `invoices:read`, `contacts:read`,
     `documents:read` (contrôlé contre la matrice de `sec:mcp-outils`).
   - **P5** — un **prompt standard** (`kesh-assistant` ou son nom réel, relu à l'étape 0), en trois sous-gestes :
     (a) l'Administrateur écrit une **version personnalisée** à l'écran des prompts (32-10b, *Enregistrer comme nouvelle
     version*), texte noté ; (b) il constate qu'elle est **en vigueur** (pastille *Personnalisé*) — une version écrite
     par l'Administrateur n'a pas d'approbation, *Approuver* ne portant que sur une version proposée par un agent (fiche
     32-10b, AC-7, l. 328-343, et AC-9, l. 384-395 ; à relire au code à l'étape 0, et P5b réécrit « l'approuver » si le
     code l'exige) ; (c) R04 liste le standard, R05 en rend le texte, qui se compare à celui de (a). La forme sous
     laquelle **Claude Code** propose le prompt (observation (g)) n'est pas de ce geste : recette de Guy (AC-A8).
   - **P6** — R06 : la demande du bilan de l'exercice de la démonstration (`<X>` lu en P0 (e) ; le nom de l'argument
     relu à la fiche 32-9, l. 356, et au code à l'étape 0). Sous la clé requise, la réponse est l'accusé « envoi
     n° `<n>` en attente » — **aucun montant ne part** (AC-A5 (b)). **Un envoi** est déposé, par construction : R06
     est le seul appel d'un outil de données du parcours.
   - **P7** — *Administration → Envois du serveur MCP* : l'envoi `<n>` **modifié** puis envoyé — **une ligne changée**,
     qui ne porte ni 20 432.05 ni 17 345.10 (l'intitulé, par exemple), texte envoyé noté ; captures. La liste des envois
     porte l'envoi `<n>`, et lui seul.
   - **P8** — R07 : la relecture de l'envoi `<n>` ; la réponse porte le texte validé en P7 (AC-A5 (c)). Combien de fois
     **un agent** relit un envoi en attente (observation (d)) n'est pas de ce geste : recette de Guy.
   - **P9** — la trace : *Traçabilité* filtrée sur la période du parcours — les lignes « Outil du serveur MCP appelé »,
     les décisions d'envoi, la consultation du bilan (`report.generated`, #776) ; le journal du serveur porte la ligne
     d'admission du filtre (`DEBUG kesh_api::mcp::reseau`, client `127.0.0.1`).
   - **P10** — *Administration → Sauvegarde complète* : export du `.keshbackup` (pour E), **clé encore active**.
   - **P11** — révoquer la clé ; puis R08 et R09 : chacune reçoit **401**, dès la première requête (la clé se contrôle
     avant le corps, `transport.rs`) ; leurs en-têtes se citent (un `WWW-Authenticate` présent ou non : moitié serveur
     de l'observation (f)).

   **Les requêtes et les envois, comptés** : **neuf** requêtes au parcours (R01 à R09), **quatre** à la restauration
   (E01 à E04, AC-E2), **neuf** au rejeu T-8 (AC-A6) ; **un envoi** déposé et validé par instance — R06 et P7 au
   parcours, E03 sur la seconde instance, le R06 du rejeu sur la troisième. Aucune session de Claude Code n'est ouverte
   par l'agent.

   **Hors du parcours, décrits par le manuel tel que le code le fait, non rejoués** : une clé `dispensée` ; les portées
   `invoices:read`, `contacts:read`, `documents:read`, `prompts:write` et `proposer_prompt` ; le refus d'un envoi et son
   échéance ; le filtre d'adresse refusant ; un mandataire déclaré ; plusieurs sociétés ; le proxy de DSM et Traefik ;
   les libellés piégés (Annexe, Q3).
3. **AC-A3 — Le journal d'exécution**, `kesh-gate-logs/3211/parcours.log` : pour chaque geste P0 à P11, l'heure, la
   commande ou l'action d'écran, l'identifiant de la requête, le statut HTTP, la sortie (ou la capture qui en tient
   lieu), et le constat ; en tête, les versions de l'AC-A1. Les observations (a) à (h) de la spike § 2.4 portent sur
   **le comportement de Claude Code** : l'agent ne les tranche pas, elles se tranchent à la recette de Guy (AC-A8). Le
   journal note seulement ce que **le serveur** en montre : la `protocolVersion` répondue à R01 (moitié serveur de
   (h)), l'absence de `Mcp-Session-Id` (moitié serveur de (a)), les en-têtes du 401 de R08 (moitié serveur de (f)).
4. **AC-A4 — Les pièces de preuve** (hors dépôt, `kesh-gate-logs/3211/`) : `requetes/`, `reponses/` (corps et en-têtes),
   `amorcage/`, les captures d'écran, thème clair, numérotées (`captures/NN-<geste>.png`) ; l'export CSV du journal
   d'audit de la période (*Traçabilité*). Aucune pièce ne contient de donnée réelle : la démonstration seule
   (`~/kesh-test/` n'est pas ouvert). `cle-parcours.entete` et `amorcage/cle-amorcage.entete` portent le secret d'une clé de démonstration, révoquée en P11 et en P0 (d). L'export `.keshbackup` de P10
   (fichier secret) s'écrit sous `kesh-gate-logs/3211/export/`, hors dépôt, supprimé avec les bases.
5. **AC-A5 — Ce que le parcours doit établir**, chaque point par une sortie citée au Dev Agent Record :
   (a) R03 liste exactement les outils de P4 ; (b) **avant P7, aucune réponse de `/mcp` ne contient de montant du
   bilan** : les deux montants de P0, **20 432.05** et **17 345.10**, cherchés par `grep -cE` dans `reponses/R01.json` à
   `reponses/R06.json` → **0** chacun ; le motif s'écrit après lecture du format des montants dans le texte de
   `kesh_bilan` (séparateur de milliers et décimal, code de la 32-9 relu à l'étape 0) **et de son échappement JSON**
   (une apostrophe typographique peut s'écrire `’` dans le corps), et **un témoin positif** le valide : le **même**
   `grep` sur `reponses/R07.json` (la relecture, qui porte le texte validé) rend **au moins 1** pour chacun — sans quoi
   le contrôle passe à vide et (b) **n'est pas établi** : constat au Dev Agent Record, et le motif se corrige avant de
   conclure. La capture de P0 (e) montre les mêmes montants à l'écran ; (c) **le bloc texte de données de la réponse à
   R07** (le second bloc de `result.content`, `mcp/reponse.rs:60-70` ; le premier est le texte de relecture de Kesh) **est,
   octet pour octet, celui validé en P7**, modification comprise (chaîne JSON décodée, outil de décodage nommé au
   journal, comparée au texte noté) ; (d) les lignes du journal d'audit de P9 existent, au nom du créateur de la clé, et la consultation du
   bilan s'y confond avec une consultation à l'écran tant que #776 est ouverte (constat, non défaut de la story) ;
   (e) R08 et R09 → 401.
6. **AC-A6 — Le rejeu depuis le PDF** (T-8), **un seul**. Un second passage de P0 à P11, sur une démonstration neuve
   (base `kesh_3211_t8`, troisième instance sur `P` une fois la première arrêtée) : P0 et les neuf requêtes **depuis
   cette fiche** (le PDF décrit Claude Code, que l'agent ne lance pas — déclaré au journal), les **gestes d'écran** P1,
   P2, P5 (a) (b), P7, P9, P10, P11 **en suivant le PDF** de *Brancher un agent* (AC-C2) et non le journal ; tout
   sous le sous-dossier propre `kesh-gate-logs/3211/t8/` (journal `parcours-t8.log`, `requetes/`, `reponses/`,
   `amorcage/`, `captures/`, `cle-parcours.entete`, export), dossiers `/tmp/kesh-e2e-3211-t8/…` : rien du premier passage
   n'est réécrit. Toute différence entre le PDF et ce qu'il faut
   faire **corrige le manuel** (puis PDF régénéré) ; ces corrections n'appellent pas de troisième passage. Les gestes
   de Claude Code du PDF se rejouent à la recette de Guy (AC-A8). Un écart qui révélerait un défaut du produit suit
   l'AC-A7.
7. **AC-A7 — Un défaut révélé ne se corrige pas ici** (C-32-11-3). Si le parcours ou la recette de Guy révèle un défaut
   du produit — un refus inattendu, une réponse qui s'écarte d'une clause, un texte de la poignée qu'un agent ne suit
   pas (C-32-8-26) — la 32-11 **ne touche pas** au code de production : elle écrit le constat au journal et au Dev Agent
   Record, **demande l'issue** à l'orchestrateur, et le manuel décrit le comportement **réel** (ou se tait sur le geste).
   **Si le défaut empêche le parcours** (une requête de P4 à P8 ne reçoit pas la réponse que la fiche attend, aucun
   envoi ne se dépose), l'agent **s'arrête** et le dit à l'orchestrateur, avec le constat : la correction est une story
   à part, avant la release (Q4).
8. **AC-A8 — La recette de Guy : Claude Code, réellement** (C-32-11-29, point 1). Ce que l'agent ne joue pas — tout geste
   de Claude Code : `claude mcp add --scope local|user`, la commande `/mcp`, la liste des outils dans le client, un
   prompt invoqué par sa commande (forme documentée `/kesh-demo:<prompt> (MCP)`, spike l. 253, observation (g)), un
   outil appelé, ce que l'agent dit d'un envoi en attente et sa relecture — reste marqué **« non éprouvé »** au manuel,
   geste par geste, avec ce qu'il faut observer. Guy le joue à la recette de la 0.16.0, **depuis le PDF**, sur une
   démonstration (jamais ses livres), en vingt à trente minutes. **La capture**, en une étape, au début de cette
   recette : `socat -v TCP-LISTEN:<port de capture>,bind=127.0.0.1,reuseaddr,fork TCP:127.0.0.1:<port de Kesh> 2>
   /home/gcorbaz/devel/kesh-gate-logs/3211/capture-recette.log`, et l'URL du client configurée sur le port de capture.
   Si Claude Code envoie une `Origin` que Kesh refuse (403), la capture garde la requête refusée, qui répond à
   l'observation (b) ; Guy pointe alors le client sur le port de Kesh pour finir, sans capture. La capture sert à
   **trancher les observations (a) à (h)** et à **comparer les requêtes réelles aux clauses** de K1 à K3, K5 à K7 et
   K14 — comparaison faite par l'orchestrateur, sortie citée ; **une contradiction s'ouvre en issue** (AC-A7), elle ne
   change pas le contrat de la CI (AC-B5). La capture porte, à `Authorization: Bearer`, le secret de la clé de démonstration : le masquer avant tout partage, elle
   reste hors du dépôt. Les marques « non éprouvé » ne se retirent qu'après cette recette, par
   l'orchestrateur, reporté au Change Log. **Il écrit aussi, à la release et après la recette, la phrase du CHANGELOG
   sur Claude Code** (l'entrée de l'AC-D4 ne le nomme pas) ; **la version de Claude Code se relève à la recette**
   (`claude --version`, par Guy), pour la mention « observé avec Claude Code 2.1.x » de l'AC-C2. `claude mcp add` **ne s'éprouve pas** par un agent (il écrirait la
   configuration de Guy).

### B — Le test de contrat

9. **AC-B1 — Un binaire neuf, `crates/kesh-api/tests/mcp_contrat.rs`** : `#[sqlx::test]`, routeur de production par
   `tests/common/mcp.rs` (pair simulé par `MockConnectInfo`, `oneshot`), **aucun réseau** ; il tourne en CI par le job
   backend (fait 4), ce que le rapport final demande à l'orchestrateur de constater au journal de la CI de la PR (nom du
   binaire et nombre de tests).
10. **AC-B2 — L'oracle est la spécification, jamais la sortie de Kesh** (C-32-11-7, C-32-11-27). Chaque cas porte
    l'une de deux marques, et chaque **assertion** se range sous l'une d'elles :
    - **`CLAUSE`** — l'URL de la page de la révision `2025-11-25` (date dans l'URL, comme la spike § 1.1), la section,
      et la **phrase citée**, portant **MUST** (ou **MUST NOT**), qui fonde l'attendu ; l'attendu s'écrit **depuis cette
      phrase** : ce que la spécification **exige** (clés requises et leurs types, valeurs imposées, forme générale de
      `CallToolResult` — `content`, tableau de blocs portant chacun un `type`).
    - **`DECISION_KESH`** — ce que la spécification laisse au serveur et que Kesh a décidé, avec sa référence
      (C-32-2-x, C-32-6-x, C-32-8-x, ou la story qui l'a posé, relue à l'étape 0) : la **liste close** des clés de
      `InitializeResult` (K1), de `Tool` (K7), de `Prompt` (K14) ; la version la plus récente sur une demande non servie
      (K3) ; des blocs de **texte** seulement (K8 à K10, K14) ; la **présence** d'`isError` (K8) ; **l'absence de
      session** (`Mcp-Session-Id`, AC-B3 ; la spécification dit « MAY assign a session ID », spike § 1.1) ; la forme de
      l'accusé et de la relecture (K10, 32-8 : `structuredContent {envoi, etat}`, second bloc de texte,
      `mcp/reponse.rs:44-75`) ; `-32602` pour un outil inconnu (K11) et, pour un outil hors portée, une réponse
      identique à l'inconnu (K12) ; le **corps** des refus
      (K15 sans `id`, K17 en texte et sans `-32022`, K21 `-32600` sans `id`) ; le 401 d'une clé `kesh_mcp_` (K16) ;
      l'en-tête `MCP-Protocol-Version` absent admis (K18) ; lot refusé en 400 (K19) ; `DELETE` en 405 et `Allow: POST`
      (K20) ; le 400 d'un message de type réponse (K21) ; `readOnlyHint: true` (K7) ; 413 au-delà de 1 Mio (K22).
    **Rangement** : sont `CLAUSE` les assertions que la spike cite en MUST (K15 le 403, K17 le 400) (**K5 et K20 le `GET`** n'y sont pas : la spike en cite la forme sans MUST, ils sont `DECISION_KESH`
    « à retrouver en MUST sur la page », étape 0) et les exigences de base que la fiche nomme — l'enveloppe JSON-RPC 2.0 (AC-B3), la version renvoyée telle
    quelle quand elle est servie et une version servie sinon (K1 à K3), les clés requises d'un résultat (K1, K7, K8,
    K14), la réponse vide à `ping` (K6), `-32601` pour une méthode inconnue (K13, JSON-RPC 2.0 § 5.1) —, chacune **à
    citer** à l'étape 0 : si la page ne la porte pas en MUST, elle passe sous `DECISION_KESH`. **Tout le reste** — ce
    que la spike dit permis, un SHOULD, un exemple (« e.g. »), ce que la fiche ne tient que de mémoire (types de bloc,
    `role`, `-32602` d'un outil inconnu, la page d'autorisation, le 202 de K21) — se range sous `DECISION_KESH`, marqué
    « à constater à l'étape 0 sur la page », et ne passe sous `CLAUSE` qu'à l'étape 0, sur une phrase citée en MUST.
    **Rien de douteux ne s'écrit `CLAUSE`.** Une assertion `CLAUSE` que la page dément se réécrit selon la page. Une assertion
    `DECISION_KESH` ne se présente jamais comme clause : le jour où une révision ajoute un champ, le test dit « décision
    de Kesh », non « écart au protocole ». Ce qui touche au **schéma officiel** (clés requises, caractère facultatif
    d'`isError`, session facultative, types de bloc admis) **se constate à l'étape 0, sur la page de la spécification**,
    phrase citée.
    ⛔ **Les attendus sont des littéraux écrits dans le test** — textes d'accusé et de relecture, `INSTRUCTIONS`, codes,
    corps texte des refus, versions servies — comme `initialize_ping_prompts` le fait déjà (fait 3). **Aucun attendu
    n'est recopié d'une réponse capturée de Kesh, ni construit par `kesh_api::mcp::textes`, ni par les aides
    `attendu_*` de `tests/common/mcp.rs`** (fait 3) : un contrat dont l'attendu sort du serveur testé est vert par
    construction (`CLAUDE.md` § *Inventorier les sites NON RÉSOLUS*, D4-ter). La preuve n'est pas une garde de texte
    (fiche, en-tête) : ce sont les mutations **M17**, **M18** (K10), **M21** (K1) et **M22** (K17), qui changent un mot
    des textes de production et doivent faire rougir leur cas — elles passeraient à vide si l'attendu venait de
    `mcp::textes`. `grep -nE 'attendu_|\btextes\b' crates/kesh-api/tests/mcp_contrat.rs` → aucune ligne de code (une
    ligne de commentaire rendue se cite et se justifie), sortie citée au Dev Agent Record (contrôle de revue, non
    garde) — la forme `use kesh_api::mcp::{textes::…}` y est prise.
11. **AC-B3 — L'enveloppe JSON-RPC, sur chaque réponse JSON du binaire** : une fonction
    `enveloppe_conforme(rendu: &Rendu, id_attendu: Option<i64>)` (`Rendu`, en-têtes comprises, `tests/common/mcp.rs:252`),
    appelée par **chaque cas dont la réponse est JSON** ; l'assertion d'en-tête (`DECISION_KESH` ci-dessous) se fait sur
    **chaque** `Rendu`, JSON ou non (K5 vide, K17 et K22 en texte, K20 vide compris). `CLAUSE` : `"jsonrpc": "2.0"` ;
    `Some(id)` exige l'`id` **présent et égal** ; `None` n'est admis que pour une erreur de transport sans `id`
    (C-32-6-42 : K15, K16, K19, K21 selon leur corps), et exige alors l'`id` absent ou `null` selon la clause relue ;
    **exactement un** de `result` et `error` ; `error.code` entier, `error.message` chaîne. `DECISION_KESH` : **aucun
    en-tête `Mcp-Session-Id`** (serveur sans état, spike § 1.3).
12. **AC-B4 — Les cas, liste close** (C-32-11-8) — chacun avec ses assertions `CLAUSE` et `DECISION_KESH` :

    | # | requête | attendu |
    |---|---|---|
    | K1 | `initialize`, `protocolVersion: "2025-11-25"` | 200 JSON ; `CLAUSE` : les clés **requises** de `InitializeResult`, relues à l'étape 0, présentes et de bon type (`protocolVersion` = `"2025-11-25"`, la version demandée et servie, *lifecycle* ; `capabilities`, `serverInfo` avec `name` et `version`) ; `DECISION_KESH` : la liste **close** des clés, et `instructions` égal au **littéral** du test |
    | K2 | `initialize`, `"2025-06-18"` | `CLAUSE` : `protocolVersion` = `"2025-06-18"` |
    | K3 | `initialize`, `"2024-11-05"` | `CLAUSE` : `protocolVersion` est **l'une des versions servies**, écrites en littéral dans le test (`"2025-11-25"`, `"2025-06-18"`) — le serveur répond une version qu'il sert (spike l. 161-163) ; `DECISION_KESH` : `"2025-11-25"`, la plus récente (à constater à l'étape 0 sur la page : un SHOULD la laisse ici) |
    | K4 | *retiré à la validation P2* | aucune capture de Claude Code n'est faite par l'agent (AC-B5) : aucune requête réelle à rejouer ; le numéro reste, pour ne pas renuméroter |
    | K5 | `notifications/initialized` | `DECISION_KESH` : 202, corps vide (à retrouver en MUST sur la page, étape 0 : alors `CLAUSE`) |
    | K6 | `ping` | `CLAUSE` : `result` = `{}` |
    | K7 | `tools/list` (clé `books:read`) | `CLAUSE` : chaque outil a `name` et `inputSchema` de `type` `"object"` ; `DECISION_KESH` : `annotations.readOnlyHint` = `true`, liste close des clés de `Tool`, aucun outil hors portée |
    | K8 | `tools/call` `kesh_version`, clé `dispensée` | `CLAUSE` : `result.content` est un tableau, chaque bloc porte un `type` ; `DECISION_KESH` : des blocs `type` `"text"` seulement, `isError` présent et faux |
    | K9 | `tools/call` d'un **outil de données** (32-9a), clé `dispensée` | idem K8 ; le texte porte la donnée posée par le test |
    | K10 | le même appel, clé **requise** | `CLAUSE` : forme de `CallToolResult` (comme K8) ; `DECISION_KESH` (32-8, AC-B5 et AC-B6) : l'accusé égal au **littéral** du test, aucune donnée ; puis l'envoi décidé **par la route de décision** (`POST /api/v1/admin/mcp/sends/{n}/decision`, avec `session` et `requete_api` de `tests/common/mcp.rs:666`, `:685` ; précédent `mcp_envois_e2e.rs:86`), `kesh_lire_envoi` rend le texte de relecture égal au **littéral** du test, et le second bloc égal à la chaîne que le test a envoyée à la décision |
    | K11 | `tools/call` d'un outil inconnu | `DECISION_KESH` : `error.code` = `-32602` (la page *tools* le donne en exemple d'erreur de protocole, de mémoire : à constater à l'étape 0 sur la page — une phrase en MUST le promeut `CLAUSE`) |
    | K12 | `tools/call` d'un outil hors portée | `DECISION_KESH` (C-32-6-21) : **identique** à K11, corps pour corps |
    | K13 | méthode non servie (`resources/list`) | `CLAUSE` : `error.code` = `-32601` |
    | K14 | `prompts/list` puis `prompts/get` d'un standard (32-10a) | **témoin** : `prompts[]` **non vide** et portant le nom du standard, relu à l'étape 0 (sans quoi l'assertion suivante passe à vide, M20 ; les standards s'amorcent au démarrage, non dans le routeur de test — § *À recontrôler*) ; `CLAUSE` : chaque prompt porte `name`, chaque message de `messages[]` porte `role` et `content` ; `DECISION_KESH` : liste close des clés de `Prompt`, `content.type` = `"text"`, `role` ∈ {`user`, `assistant`} (à constater à l'étape 0 sur la page : une énumération citée en MUST le promeut `CLAUSE`) |
    | K15 | `Origin` étrangère | `CLAUSE` : 403 (« MUST respond with HTTP 403 Forbidden », spike l. 85-87) ; `DECISION_KESH` (C-32-6-42) : corps JSON-RPC sans `id` (la spike : « permis ») |
    | K16 | sans `Authorization` | `DECISION_KESH` : 401 — la clé `kesh_mcp_` est hors de l'autorisation OAuth de MCP (aucun `WWW-Authenticate`, observation (f) de la spike) ; référence de la décision relue à l'étape 0 ; à constater sur la page d'autorisation : rien n'y passe sous `CLAUSE` sans phrase en MUST qui vaille pour un serveur sans OAuth |
    | K17 | `MCP-Protocol-Version: 2026-07-28` | `CLAUSE` : 400 (« invalid or unsupported […] MUST respond with `400 Bad Request` », spike l. 96) ; `DECISION_KESH` (C-32-2-1) : corps texte égal au **littéral** du test, **pas** de `-32022` |
    | K18 | `MCP-Protocol-Version` absent | `DECISION_KESH` : admis (même réponse que K6) — la page dit « SHOULD assume protocol version `2025-03-26` » (spike l. 95-96) : un SHOULD n'est pas une clause |
    | K19 | lot JSON-RPC (`[…]`) | `DECISION_KESH` : 400 (la clause — « MUST be a single JSON-RPC […] », spike l. 88-89 — oblige le client ; la réponse à un lot est une décision) |
    | K20 | `GET /mcp` ; `DELETE /mcp` | `DECISION_KESH` : `GET` → 405 (« or else return HTTP 405 Method Not Allowed », spike l. 92, sans MUST ; à retrouver en MUST sur la page, étape 0 : alors `CLAUSE`) ; `DECISION_KESH` : `DELETE` → 405, en-tête `Allow: POST` sur les deux |
    | K21 | message de **type réponse** (`result` sans `method`) | `DECISION_KESH` (C-32-6-42, report B4 de la 32-6, `32-6-endpoint-mcp.md:1304-1308`) : 400, corps JSON-RPC `-32600` sans `id` ; à constater à l'étape 0 sur la page *transports* : si elle porte, pour une entrée que le serveur n'accepte pas, « 202 Accepted » ou « an HTTP error status code (e.g., 400) », une assertion `CLAUSE` s'ajoute — « 202 sans corps, **ou** un statut d'erreur 4xx », phrase entière citée —, jamais « 400 » seul |
    | K22 | corps de 1 Mio + 1 octet | `DECISION_KESH` : 413 |

    Les cas K1 à K22 **ne remplacent ni ne modifient** les tests existants (fait 3) : ils en recoupent une partie,
    rangée par clause (C-32-11-8).
13. **AC-B5 — Aucune fixture de Claude Code à la 32-11** (validation P2, C-32-11-28, C-32-11-29). Le contrat de la CI
    repose sur les **clauses citées** ; il ne dépend d'aucune capture, et l'agent n'en fait aucune. Les requêtes réelles
    de Claude Code se capturent à la **recette de Guy** (AC-A8) et se comparent aux clauses de K1 à K3, K5 à K7 et K14 ;
    une contradiction s'ouvre en issue. Versionner une fixture tirée de cette capture, si Guy le veut, est une issue à
    part. K4 et sa mutation MF1 sont **retirés** ; `crates/kesh-api/tests/fixtures/mcp-contrat/` n'est pas créé.
14. **AC-B6 — Les mutations** (§ *Tests et mutations*) : M1 à M22, chacune tuée par le cas nommé.

### C — Les manuels

15. **AC-C1 — La description détaillée du fonctionnement de MCP, manuel d'administration** (C-32-11-11) : une
    sous-section `\subsection{Comment fonctionne le serveur MCP}\label{sec:mcp-fonctionnement}`, **première** de
    `sec:mcp` (32-7). Elle dit, chaque affirmation relue au code à l'étape 0 :
    (i) **ce qu'est MCP** — un protocole par lequel un client (l'agent) appelle des **outils** et demande des **prompts**
    à un serveur ; JSON-RPC sur HTTP (`POST /mcp`), révision `2025-11-25` (`2025-06-18` acceptée), sans session ;
    (ii) **le parcours d'une requête**, en étapes numérotées dans l'ordre de `mcp/mod.rs:11-34` relu à l'étape 0 : pair
    TCP et filtre d'adresse (renvoi `sec:mcp-journal-filtre`), méthode, `Origin`, `Content-Type`, version, clé et portées,
    société, corps, répartiteur, outil sur une connexion en lecture seule, validation avant envoi (renvoi
    `sec:mcp-envois`), journal d'audit, réponse — chaque refus avec la réponse que voit le client (renvoi
    `sec:mcp-erreurs`) ; (iii) **les portées**, et le renvoi à la matrice de `sec:mcp-outils` (jamais une seconde copie
    de la matrice) ; (iv) **les prompts** (renvoi `sec:mcp-prompts`) : un prompt est choisi par l'utilisateur dans le
    client, non imposé à l'agent ; (v) **la validation avant envoi**, par clé, pour les **outils de données** —
    `proposer_prompt` n'y passe pas (fiche 32-10a, l. 821, 837) ; (vi) **ce que Kesh garantit et ce qu'il ne garantit
    pas** — **aucune garantie générale de « lecture seule »** : les outils de **lecture** s'exécutent sur une `Lecture`
    à connexion `READ ONLY`, tenue **par l'application**, non par un utilisateur MariaDB limité (Q11 de la spike : les
    mots « par construction » ne s'écrivent pas) ; **seul `proposer_prompt`** (portée `prompts:write`) écrit — une
    version de prompt « à valider », jamais dans les livres, hors de la validation avant envoi —, et **sa trace est la
    validation de cette version par l'Administrateur** à l'écran des prompts (renvoi `sec:mcp-prompts`) ; chaque
    affirmation relue au code fusionné de la 32-10a (§ *À recontrôler*) ; un appel **écrit** (dernière utilisation, journal) ; le filtre d'adresse n'est pas un pare-feu ; une clé
    `dispensée` envoie sans montrer ; une clé d'API donnée au même agent lit sans validation (`:2963`) ; l'enveloppe
    « donnée non fiable » **aide** l'agent à ne pas suivre un libellé, elle ne l'en empêche pas ; ce qui part chez le
    fournisseur du modèle n'est plus dans Kesh ; et les limites de l'AC-C6.
16. **AC-C2 — *Brancher un agent***, `\subsection{Brancher un agent}\label{sec:mcp-brancher}` sous `sec:mcp`, après
    `sec:mcp-client` : les **gestes d'écran** P1, P2, P5 (a) (b), P7, P9, P10, P11 **tels que joués**, libellés et
    messages copiés du journal — P0 n'y est pas un geste, une phrase dit seulement que le parcours a été joué sur une
    démonstration où quelques écritures avaient été saisies —, chaque capture décrite en mots (aucune image au
    manuel) ; **le côté du client** — brancher Claude Code, lister les outils, invoquer un prompt, demander une donnée,
    relire l'envoi validé — par **renvoi** à `sec:mcp-client` pour la configuration, sans seconde copie (C-32-7-22),
    et, pour la suite, en gestes de Claude Code **marqués « non éprouvé »** (AC-A8) jusqu'à la recette de Guy ; **ce
    que l'agent reçoit** d'un envoi en attente puis validé — les textes de l'accusé et de la relecture, tels que R06 et
    R07 les ont rendus — et ce qu'on fait à l'écran ; un paragraphe *Après une restauration* (AC-E3). Les observations
    de Claude Code (a) à (h) n'y entrent qu'après la recette de Guy, **datées et rapportées à la version de Claude
    Code** (« observé avec Claude Code 2.1.x, le … »), par l'orchestrateur.
17. **AC-C3 — `sec:mcp-client` amendée en place** (C-32-7-22) : la mention « non éprouvée » y **reste** — la 32-11 ne
    joue aucun geste de Claude Code ; elle se retire geste par geste après la recette de Guy (AC-A8) ; ce que le
    parcours HTTP réfute (une réponse du serveur que la section décrit autrement) se corrige ; **aucune mention de
    Hermes** — si la 32-7 en a laissé une, elle se retire (C-32-11-18).
18. **AC-C4 — Le compte des routes**, `admin-manual.tex` (l. 2978 à `ed981c7d`, relue) : « N des M routes à verbe
    mutant » et « N + 4 + 2 = M » **recomptés depuis `audit_route_registry.rs`** au dernier état (après la 32-10b),
    commande et sortie citées ; la phrase des exceptions (quatre exemptées, deux sans effet) relue contre le registre ;
    tenu ensuite par la garde de l'AC-T1.
19. **AC-C5 — Le renvoi `admin-manual.tex:2926`** (« MCP » dans le guide d'intégration) : il nomme désormais le serveur
    intégré (`\S\ref{sec:mcp}`) et ne laisse pas croire que l'intégration MCP passe par l'API.
20. **AC-C6 — Les limites connues aux manuels** — chacune **seulement si son issue est ouverte à l'étape 0**, et **par
    renvoi** si la section d'une autre story le dit déjà (aucune seconde copie), le numéro d'issue cité :

    | issue | ce que le manuel dit | où |
    |---|---|---|
    | #776 | une consultation de rapport par l'agent se confond, au journal, avec une consultation à l'écran ; la ligne « Outil du serveur MCP appelé » de même heure permet de les rapprocher | `user-manual.tex` § *Traçabilité* (amender `:3153-3157`), `sec:agent-mcp`, `sec:mcp-fonctionnement` (vi) |
    | #759 | un appel abandonné par le client va au bout côté serveur ; un porteur de clé qui coupe en boucle peut faire attendre les autres appels | `sec:mcp-fonctionnement` (vi) |
    | #765 (5) | le texte d'un envoi échu reste en base **et dans une sauvegarde** jusqu'à la purge suivante | **déjà dit** (fait 13 : `admin-manual.tex:2238` et `:2949` à `ed981c7d`) — rien, sauf si la 32-7 a déplacé ou retiré ces phrases (relu à l'étape 0) |
    | #768 (1) | une décision d'envoi peut être annoncée en échec alors qu'elle est enregistrée : recharger | `sec:mcp-envois` (non dit à `ed981c7d`) |
    | #773 (8) | la liste des envois montre les 200 plus récents | `sec:mcp-envois` (non dit à `ed981c7d`) |
    | #773 (9) | la purge n'a lieu qu'au dépôt suivant | **déjà dit** (`:2949`) — rien |
    | #770 (5) | derrière le proxy de DSM, ce que voit le filtre n'est pas éprouvé | renvoi `sec:mcp-journal-filtre`, qui le dit |
    | #761 | rien (tests seulement ; son point 7 est l'AC-C4) | — |

21. **AC-C7 — La description détaillée, manuel utilisateur** (C-32-11-12) : une section
    `\section{L'agent IA et le serveur MCP}\label{sec:agent-mcp}`, après *Traçabilité* (`:3108`) et avant *FAQ*
    (`:3222`), à l'adresse d'un Comptable : ce qu'est MCP, en mots ; **ce que l'agent peut lire** selon les portées de sa
    clé (renvoi au manuel d'administration, § *Les outils et leurs portées*, par son titre — un `\ref` ne traverse pas les
    manuels) ; **ce qu'on montre avant l'envoi** — un Administrateur voit le texte exact, l'envoie tel quel, le modifie
    ou le refuse ; une clé dispensée envoie sans montrer ; **invoquer un prompt** dans Claude Code, sous la forme que sa
    documentation donne (`/kesh-demo:<prompt> (MCP)`, spike l. 253), marquée « non éprouvé » jusqu'à la recette de Guy
    (AC-A8) ; **reconnaître l'activité de l'agent** au journal (et #776) ; ce que Kesh garantit
    et ne garantit pas, en quatre ou cinq phrases, renvoyant au manuel d'administration pour le détail. **Aucune phrase
    ne promet une écriture** : l'agent ne modifie pas les livres ; il peut proposer un prompt, qu'un Administrateur
    approuve ou refuse.
22. **AC-C8 — La brochure** : `marketing-brochure.tex:480` perd « MCP server » (reste « Webhooks pour assistants IA
    conversationnels », ou la ligne se réécrit sans MCP) ; la liste livrée (`:390-400`) gagne une ligne « Serveur MCP
    intégré : un agent IA du réseau local **lit** les livres, désactivé par défaut, clés dédiées à portées fines,
    données montrées avant l'envoi sous une clé à validation requise. *(v0.16.0)* » — sans promesse d'écriture.
23. **AC-C9 — Les PDF.** `admin-manual.pdf`, `user-manual.pdf`, `marketing-brochure.pdf` régénérés (`make fr` dans
    `docs/manual/`, sous `flock …/.gate.lock`) et commités. Contrôle **aplati**, motif par motif, sans `-i`, commande et
    sortie citées au Dev Agent Record, **sous une locale UTF-8 déclarée** : `LC_ALL=fr_FR.UTF-8 pdftotext
    docs/manual/fr/<m>.pdf - | tr '\n' ' ' | tr -s ' ' | grep -cE '<motif>'` — sur une ligne aplatie, `grep -c` dit la
    **présence** (0 ou 1), non un nombre d'occurrences. **Règle du choix** (reprise de la 32-7, AC-21, l. 887-889 de sa
    fiche) : **chaque motif rend 0 sur le PDF de la base** (celui d'`origin/main` à l'étape 0) **et 1 au PDF
    régénéré** ; le développement cite les deux sorties, à l'étape 0 et au dernier commit ; **un motif que la base rend
    déjà se remplace avant de servir** par un motif propre au texte neuf. Motifs prévus — admin : `Comment fonctionne
    le serveur MCP`, `Brancher un agent`, `2025-11-25`, `#776`, et le compte « N des M » recompté (si la 32-10b l'a déjà
    écrit juste, la base le rend : l'AC-C4 est alors déclarée « déjà vraie », sortie citée, et ce motif ne prouve
    rien) ; utilisateur : `L.{1,3}agent IA et le serveur MCP` (l'apostrophe typographique `’` occupe trois octets : sous
    `LC_ALL=C`, `L.agent` rend 0 — constat de la validation P1, R6), `#776` ; brochure : `Serveur MCP intégré`, et
    `MCP server` → **0** au PDF régénéré (motif d'absence : 1 à la base). Le motif `Outil du serveur MCP appelé`, que les
    PDF de la base rendent déjà (validation P2, R4 : admin 1, utilisateur 1), **n'en est pas**. Relevé de la validation
    P2 à `ed981c7d`, à refaire sur la base de l'étape 0 : `Comment fonctionne`, `Brancher un agent`, `2025-11-25`,
    `#776`, `Serveur MCP intégré`, `L.{1,3}agent IA et le serveur MCP` → 0. **Césure** (même règle) : un motif que la
    césure coupe dans le PDF aplati (`2025-11- 25`) se raccourcit (`2025-11-`), et l'écart se déclare. Aucun `??`
    (référence non résolue) dans les trois PDF aplatis ; aucun caractère U+FFFD.

### D — Les textes publics

24. **AC-D1 — `docs/api-external.md` § 6.4** (C-32-11-14) : réécrit — Kesh a son **propre serveur MCP** depuis la
    0.16.0, sur `/mcp`, avec ses clés `kesh_mcp_` (renvoi au manuel d'administration, § *Le serveur MCP*) ; une clé
    d'API (PAT) y est refusée ; l'ancien conseil (serveur MCP générique avec une clé `read`) **reste possible** mais passe
    **sans** la validation avant envoi ni le journal des appels MCP, ce que le texte dit en une phrase ; l'exemple JSON à
    `baseUrl` `/api/v1` est retiré ou marqué comme celui de cette voie. **§ 9** : « serveur MCP Kesh-natif » sort de la
    phrase « Hors périmètre » (`:831`) ; rien d'autre du § 9 ne change.
25. **AC-D2 — `README.md`, ligne v0.16.0** (l. 233 à `ed981c7d`, recomptée) : trois affirmations réécrites
    (C-32-11-15) — « **en lecture seule** » devient « **en lecture seule des livres** » (`proposer_prompt` écrit une
    version de prompt, AC-C1 (vi)) ; « les données montrées à l'Administrateur avant de partir à l'agent, sous une clé à
    validation requise (le défaut) ; une clé dispensée les envoie sans les montrer, chaque envoi journalisé » ;
    « l'agent n'écrit rien dans les livres ; il peut proposer un prompt, qu'un Administrateur approuve ou refuse ; ses
    propositions d'écritures sont prévues pour la 0.17.0 ». Le statut reste **🚧 En cours** ; la section
    *Fonctionnalités* et le ✅ sont à la release (C-32-11-16).
26. **AC-D3 — `website/roadmap.html`** : `grep -n -i mcp website/*.html`, sortie citée — à `ed981c7d`,
    `roadmap.html:43`, `:223`, `:372`, `:375`. **La ligne de l'E32** (l. 375, recomptée) reçoit les mêmes corrections, en
    anglais (« **read-only** » devient « **read-only on the books** » ; l'envoi montré sous une clé à validation
    requise ; l'agent n'écrit rien dans les livres et peut proposer un prompt) ; « Planned, not yet released » reste.
    **La ligne 223** (le paragraphe d'en-tête de la feuille de route : « a built-in, read-only MCP server for AI agents
    (v0.16, in progress) ») est **prise en charge** pour la même raison, et pour elle seule : « a built-in MCP server
    that reads the books for AI agents (v0.16, in progress) » ; le reste de la ligne ne change pas. **La ligne 43** (le
    paragraphe d'introduction) porte la même formule — « a built-in, read-only MCP server for AI agents on the local
    network (v0.16, in progress) » (`sed -n 43p website/roadmap.html`, validation P2) — : **cette seule formule** reçoit
    la même correction ; le reste de la ligne, qui dit aussi « Kesh now keeps real books… since 2026-10-23 », faux
    depuis la précision de Guy du 2026-10-25, est **laissé** à l'audit de release (point 5 pour Guy) : ce défaut n'est
    pas celui de la 32-11.
27. **AC-D4 — CHANGELOG**, sous `## [0.16.0] — Non publié`, `### Ajouté` : « **Brancher un agent sur Kesh** : le
    manuel d'administration décrit le fonctionnement du serveur MCP et le parcours d'administration, joué sur une
    démonstration, restauration d'une sauvegarde comprise ; un test de contrat du serveur tourne en CI ; le manuel
    utilisateur dit ce que l'agent peut lire et ce qu'on montre avant l'envoi ; `docs/api-external.md` renvoie au
    serveur intégré. » — sans date (la release la pose). **L'entrée ne nomme pas Claude Code** (décision de
    l'orchestrateur, validation P4, A1) : la phrase sur lui, « éprouvée » ou « non éprouvée », s'écrit à la release,
    par l'orchestrateur, après la recette de Guy (AC-A8).
28. **AC-D5 — Aucun texte ne promet une écriture.** `grep -n -i -E 'MCP|agent' README.md website/*.html
    docs/api-external.md` et le PDF aplati des trois manuels relus : aucune phrase ne dit que l'agent écrit, modifie ou
    crée dans les livres (sortie citée, phrases relues).

### E — La restauration jouée

29. **AC-E1 — L'export** est celui de P10 (clé active, serveur activé, prompt personnalisé de P5, l'envoi `<n>` décidé).
30. **AC-E2 — La restauration** par le chemin de l'**installation neuve** (31-1b) : une seconde instance du binaire du
    worktree, port `P_R` (AC-A1), base `kesh_3211_r` **vide**, dossiers **propres** `/tmp/kesh-e2e-3211-r/{inbox,
    documents,backup}` (`KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR`, `KESH_ADMIN_BACKUP_DIR` : la restauration et la
    sauvegarde automatique n'écrivent pas dans ceux de la première), l'assistant → *Restaurer une sauvegarde* ; puis,
    constatés à l'écran et cités au journal `kesh-gate-logs/3211/restauration.log` : le serveur MCP **activé** (réglages
    restaurés tels quels, Q7 du kickoff) ; la clé `parcours-3211` listée active ; le prompt personnalisé **et son
    historique** ; l'envoi `<n>` décidé en P7 ; le bilan de P0 à l'écran *Rapports* (20 432.05, 17 345.10) ; puis le
    client HTTP de P3, sur `P_R`, avec **la même clé** (`cle-parcours.entete`, export pris avant la révocation de P11) :
    **quatre requêtes**, écrites sous `requetes/` avant envoi, réponses sous `reponses/` — **E01** `initialize`, **E02**
    `tools/list`, **E03** `tools/call` `kesh_bilan` avec l'identifiant d'exercice `<X_R>` lu sur la seconde instance
    **à l'écran** (aucune requête de plus : l'interdit des Dev Notes ne l'admet pas ; le journal dit comment l'identifiant a été lu) (dépôt de l'envoi `<n_R>`, validé **tel quel** à l'écran), **E04** `tools/call` `kesh_lire_envoi` `{"envoi":
    <n_R>}`, qui rend le texte validé. La révocation de P11 se joue **après** l'export, sur la première instance
    seulement ; la clé de la seconde est révoquée à la fin, et les deux bases supprimées.
31. **AC-E3 — Le manuel** : le paragraphe *Après une restauration* de `sec:mcp-brancher` dit ce qui revient (serveur
    activé s'il l'était, clés valides, prompts et historique, envois) et que le client reprend avec la même clé, en
    renvoyant à la section de la sauvegarde pour la compatibilité (« mêmes tables », `admin-manual.tex:2250`) ; aucune
    phrase de plus sur la restauration générale.

### T — La garde du compte

32. **AC-T1 — `le_manuel_dit_le_compte_du_registre`**, dans `tests/audit_route_registry.rs` (C-32-11-10) : le test lit
    `docs/manual/fr/admin-manual.tex` par `#[path = "common/manuel.rs"] mod manuel;` (normalisation `N`), y trouve
    **exactement une** occurrence de « `(\d+) des (\d+) routes à verbe mutant` » et **exactement une** de
    « `(\d+) \+ (\d+) \+ (\d+) = (\d+)` », et compare aux comptes calculés par la partition (traced, exempt, no_matter,
    total). Un nombre faux, une phrase disparue ou dédoublée → rouge, avec le compte attendu dans le message. *Retrait
    pré-déclaré* : si une passe de validation ou de revue montre un recyclage né de cette garde, elle se retire (la
    correction du compte, AC-C4, reste) — C-32-11-10. **Ce qui part avec elle** : le test `le_manuel_dit_le_compte_du_registre`,
    les mutations MT1 et MT2, la moitié « garde » de T-5 et la phrase « tenu ensuite par la garde de l'AC-T1 » de
    l'AC-C4 ; le groupe T passe à un AC (AC-T2, inchangé), la fiche à 32 AC ; `binary(audit_route_registry)` reste au
    gate ciblé (AC-T2). Le retrait s'écrit au Change Log avec ces décomptes recomptés.
33. **AC-T2 — Les gardes existantes restent vertes sans modification**, sauf inscription justifiée à une liste fermée
    (précédent C-32-6c-14) : `textes_coherents.rs` (G18 `une_installation_une_societe` sur « plusieurs sociétés », G19
    sur les URL `raw`, et deux gardes qu'une section montrant un bilan de démonstration peut toucher :
    `les_comptes_cites_en_exemple_existent_dans_les_plans_livres`, l. 411, et `la_forme_libre_nnnn_nom_est_juste_partout`,
    l. 579), `configuration_transmise.rs` (il lit `admin-manual.tex`, `MANUEL_ADMIN` l. 2619 : tout jeton `KESH_…` cité
    doit nommer une variable connue, services et volumes cités), `mcp_gardes_lexicales.rs` (les chaînes `/mcp`,
    `kesh_api::mcp` du binaire neuf), la matrice de la 32-9 contre le manuel.

## Tâches

- [ ] **T-0 — étape 0** (§ *À recontrôler*), lignes lues citées au Change Log ; arrêt si une story manque.
- [ ] **T-1 — le montage** (AC-A1) : deux ports demandés à l'orchestrateur (`P`, `P_R`) ; bases `kesh_3211`,
  `kesh_3211_r`, `kesh_3211_t8`, `kesh_e2e_3211` créées par `kesh`, migrations du worktree ; dossiers
  `/tmp/kesh-e2e-3211/…`, `/tmp/kesh-e2e-3211-r/…` et, au rejeu, `/tmp/kesh-e2e-3211-t8/…` ; dossiers de preuve
  `kesh-gate-logs/3211/export/` et `kesh-gate-logs/3211/t8/` ; build du frontend du worktree ; requêtes du parcours écrites
  sous `kesh-gate-logs/3211/requetes/` ; versions relevées.
- [ ] **T-2 — le parcours P0 à P11** (AC-A2 à AC-A5, AC-A7), journal, requêtes et réponses, captures d'écran.
- [ ] **T-3 — la restauration** (AC-E1, AC-E2) ; journal.
- [ ] **T-4 — le test de contrat** (AC-B1 à AC-B4, AC-B6) : clauses relues à la spécification `2025-11-25` (pages
  citées par la spike § 1.1, et les pages *lifecycle*, *tools*, *prompts*, *authorization* de la même révision),
  chaque assertion rangée selon l'AC-B2 ; aucune fixture (AC-B5).
- [ ] **T-5 — la garde du compte** (AC-T1) et le compte du manuel (AC-C4).
- [ ] **T-6 — les manuels** (AC-C1 à AC-C3, AC-C5 à AC-C8, AC-E3), écrits **depuis le journal** ; gestes de Claude
  Code marqués « non éprouvé » (AC-A8) ; PDF régénérés sous verrou (AC-C9).
- [ ] **T-7 — les textes publics** (AC-D1 à AC-D5).
- [ ] **T-8 — le rejeu depuis le PDF** (AC-A6), un seul ; corrections du manuel ; PDF régénérés.
- [ ] **T-9 — mutations** (§ *Tests et mutations*), une à une, restauration par `git checkout --` + `touch`.
- [ ] **T-10 — gates** (§ *Gates*), Dev Agent Record, `sprint-status.yaml` → `review`, prompt de revue P1, la liste
  des issues demandées (AC-A7), et le **mode d'emploi de la recette de Guy** (AC-A8 : la capture, les gestes de Claude
  Code du PDF, les observations (a) à (h) à trancher) remis à l'orchestrateur dans le rapport final.

## Tests et mutations prévus

Les mutations M1 à M22 portent sur le **code de production** de `mcp/` (au dernier état, relu à l'étape 0) ; chacune
doit être tuée par **le cas nommé** de `mcp_contrat.rs` (le binaire exécuté seul : `cargo nextest run -E
'binary(mcp_contrat)'`). Qu'elle rougisse aussi un test existant est permis ; qu'elle ne rougisse **que** lui ne suffit
pas (le cas du contrat passerait à vide).

| # | mutation | cas qui doit rougir |
|---|---|---|
| M1 | `initialize` répond toujours `VERSIONS_SERVIES[1]` | K1 |
| M2 | `initialize` répond la version demandée même non servie | K3 |
| M3 | `serverInfo` retiré du résultat d'`initialize` | K1 (`CLAUSE` : clé requise) |
| M4 | un champ de plus au résultat d'`initialize` (`"sessionId"`) | K1 (assertion `DECISION_KESH` : liste close), non la clause |
| M5 | un en-tête `Mcp-Session-Id` posé sur la réponse d'`initialize` — la mutation touche `reponse::rendre`, le `Resultat` n'ayant pas de champ d'en-tête | AC-B3 (assertion `DECISION_KESH`, sur le `Rendu`) par K1 |
| M6 | une notification reçoit 200 et un corps | K5 |
| M7 | `ping` rend `null` | K6 |
| M8 | `readOnlyHint` retiré d'un outil | K7 |
| M9 | `inputSchema` d'un outil sans `"type": "object"` | K7 |
| M10 | `isError` omis d'un résultat d'outil | K8 (assertion `DECISION_KESH` : présence) — mutation d'une décision, non d'une clause |
| M11 | un outil inconnu rend `-32601` | K11 (K12 reste vert : l'outil hors portée et l'inconnu passent par la même branche, ils restent égaux) |
| M12 | l'`id` de la réponse forcé à `null` | AC-B3 (K6) |
| M13 | une `Origin` étrangère rend 401 | K15 |
| M14 | une version non servie rend un corps JSON-RPC `-32022` | K17 |
| M15 | le lot traité comme son premier message | K19 |
| M16 | `politique::autoriser` rend `Autorisation::Envoi` pour `SendValidation::Required` (`mcp/politique.rs:106-111`) : la donnée part sans dépôt | K10 |
| M17 | un mot changé dans `textes::texte_envoi_depose` (`mcp/textes.rs:119`) | K10 (prouve que l'attendu de l'accusé est un littéral, AC-B2) |
| M18 | un mot changé dans `textes::texte_envoi_valide` (`mcp/textes.rs:138`) | K10 (prouve que l'attendu de la relecture est un littéral, AC-B2) |
| M19 | une méthode non servie répartie comme `tools/list` au lieu de rendre `-32601` (une branche de plus à `Methode::depuis`, `repartiteur.rs:106-116`) | K13 |
| M20 | `prompts/list` rend `[]` (les standards ignorés) | K14 (témoin : liste non vide, portant le standard) |
| M21 | un mot changé dans `textes::INSTRUCTIONS` (`mcp/textes.rs:101`) | K1 (prouve que l'attendu d'`instructions` est un littéral, AC-B2) |
| M22 | un mot changé dans `textes::CORPS_VERSION_NON_SERVIE` (`mcp/textes.rs:55`) | K17 (prouve que le corps texte attendu est un littéral, AC-B2) |

Mutations de la garde du compte : **MT1** — le manuel dit un « N » de moins → AC-T1 rouge ; **MT2** — une route du
registre passe de `Traced` à `NoMatter` (sans toucher le manuel) → AC-T1 rouge (et `the_registry_partition…`). La
mutation du témoin de la fixture, **MF1**, est **retirée** avec K4 (validation P2, AC-B5).

**Cas sans mutation, dits ici** : K2, K9, K12 (M11 le laisse égal à K11), K16, K18, K20, K21, K22, et la branche
`None` de l'AC-B3. Quatre mutations ont été ajoutées à la validation P2 (M19 à M22), au plafond fixé ; le reste est un
angle mort écrit (§ *Ce que les gardes ne voient pas*).

**Non mutable, dit ici** : l'exactitude du parcours et du manuel. Elle n'est portée que par le journal d'exécution, le
rejeu depuis le PDF (AC-A6) et la relecture en revue ; aucun test ne remplace d'avoir joué les gestes (précédent 31-4,
l. 617-619).

Tests existants qui doivent rester verts **sans modification** : tous les `tests/mcp_*.rs`, `admin_backup_e2e.rs`,
`admin_full_import_e2e.rs`, `textes_coherents.rs`, `audit_route_registry.rs` (hors l'ajout de l'AC-T1).

## Gates

- Entre les passes : gate ciblé — `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run -E 'binary(mcp_contrat) | binary(audit_route_registry) | binary(textes_coherents) |
  binary(mcp_gardes_lexicales) | binary(configuration_transmise)'` (`textes_coherents` et `configuration_transmise`
  lisent le manuel, AC-T2).
- **Au dernier commit de code** : bases propres au worktree remises à zéro (DROP/CREATE par `kesh`, **sans** redémarrer
  le conteneur, `docs/agents-orchestration.md` § 2), **gate complet** `scripts/test-fast.sh --ci` sous `flock
  /home/gcorbaz/devel/kesh-gate-logs/.gate.lock` ; frontend `check`, `lint-i18n-ownership`, `test:unit`, `build` (le
  frontend ne change pas : le build sert l'E2E) ; **E2E complet** (`docs/testing.md` § *Les échecs attendus*), port et
  base propres. « Rejouée au push » est interdit. Aucune migration, aucun dépôt touché : pas d'exception `kesh-db`.
- **Le parcours** (AC-A2, AC-E2, AC-A6) : joué **par `curl`** contre les instances de démonstration, outil déclaré ici
  (C-32-11-28) — neuf requêtes au parcours, quatre à la restauration, neuf au rejeu, chacune envoyée une fois ; aucun
  `claude`, aucun `socat`. Ce n'est pas un gate qui se rejoue au push : ses preuves sont le journal et les réponses
  gardées (AC-A3, AC-A4).
- **PDF aplatis** : AC-C9, commandes et comptes cités.
- **Recette visuelle** : la 32-11 n'ajoute ni ne modifie de contrôle visible ; ses captures (AC-A4) sont des **preuves
  du parcours**, thème clair, hors dépôt, citées au Dev Agent Record et au prompt de revue.
- Revue de code : P1 complète (Sonnet ou Opus, jamais Haiku), lentilles B, E, A ; axes nommés : **le manuel (`.tex` et
  PDF aplati) contre le journal d'exécution et contre le code**, **l'oracle du contrat (aucun attendu tiré de Kesh)**,
  **aucune promesse d'écriture**, **aucune donnée réelle, aucun secret** dans un fichier versionné.

## Ce que les gardes ne voient pas

- **Le client réel.** L'agent ne lance pas Claude Code (C-32-11-28) : ce que fait le client — les requêtes qu'il
  envoie réellement, la `protocolVersion` qu'il demande, une `Origin`, un `progressToken`, la forme du prompt, ses
  relectures d'un envoi en attente, une découverte OAuth sur un 401, la conduite d'un agent qui attend un humain — ne
  se voit qu'à la **recette de Guy** (AC-A8). D'ici là, les gestes de Claude Code restent marqués « non éprouvé ».
- **La capture de Guy ne rougit rien** : elle se compare aux clauses par l'orchestrateur, à la main ; une
  contradiction devient une issue, non un test qui échoue. Aucune fixture n'est versionnée (AC-B5).
- **Les versions futures de Claude Code.** La capture de la recette vaut pour **une** version ; une version qui demande
  une autre révision, ou passe à `2026-07-28` sans repli, n'est vue que par une nouvelle recette. Le manuel date ses
  observations.
- **La révision `2026-07-28`**, non servie (spike § 1.3) : le contrat vérifie le refus (K17), non un repli du client.
- **L'oracle écrit à la main** : les clés requises de `InitializeResult`, `Tool`, `Prompt` sont recopiées de la
  spécification par l'agent ; une lecture fausse de la spécification passe (Q2). Le schéma officiel ne le serait pas.
  Les listes closes sont des décisions de Kesh (AC-B2) : elles rougissent sur un ajout que la spécification permet.
- **Les cas sans mutation** (§ *Tests et mutations*) : K2, K9, K12, K16, K18, K20, K21, K22 et la branche `None` de
  l'AC-B3 ne sont prouvés par aucune mutation ; un de ces cas peut passer à vide sans que rien ne le dise.
- **DSM, Traefik, le NAS** (#770 (5), demande 11 de la 32-7) : le parcours se joue sur la boucle locale, sans
  mandataire. La vérification de `X-Forwarded-For` derrière le proxy de DSM se fait chez Guy, à la mise à jour du NAS ;
  la 32-11 n'en affirme rien.
- **L'image publiée** : le parcours éprouve le binaire du worktree ; l'image 0.16.0 n'existe qu'à la release, qui
  rejoue un parcours court (C-32-11-19).
- **Ce que l'agent fait d'une donnée reçue** : aucune garde ; la validation montre ce qui part, le journal le trace,
  le prompt de l'agent de développement l'interdit (32-10a) — une consigne, non une garde.
- **L'injection par libellé** : non observée par la 32-11 ; elle ne le serait que si Guy acceptait la variante de
  l'Annexe (points 1 et 3).
- **Le manuel**, hors du compte des routes : aucun gate ne le lit ; le prompt de revue nomme l'axe et exige le PDF.
- **La donnée envoyée au fournisseur du modèle** : aucune pendant le parcours de l'agent (aucun modèle n'est appelé) ;
  à la recette de Guy, la démonstration seule, ce que l'epic accepte (« données chez le fournisseur du modèle
  acceptées »).

## Dev Notes

### Le parcours, en commandes (indicatif : la liste des gestes et des requêtes de l'AC-A2 fait foi)

```sh
# terminal A — Kesh (recette du § Montage E2E local, sans KESH_TEST_MODE, port P, base kesh_3211 vide)
RUST_LOG=info,kesh_api::mcp::reseau=debug … KESH_PORT=<P> KESH_STATIC_DIR=<worktree>/frontend/build \
  KESH_INBOX_DIR=/tmp/kesh-e2e-3211/inbox KESH_DOCUMENTS_DIR=/tmp/kesh-e2e-3211/documents \
  KESH_ADMIN_BACKUP_DIR=/tmp/kesh-e2e-3211/backup DATABASE_URL=…/kesh_3211 cargo run -p kesh-api
# P0 — avec la clé d'API d'amorçage (révoquée à la fin de P0), réponses sous amorcage/
# (la clé d'amorçage, comme les autres, ne s'écrit jamais en clair : fichier d'en-tête amorcage/cle-amorcage.entete)
curl -sS -H @/home/gcorbaz/devel/kesh-gate-logs/3211/amorcage/cle-amorcage.entete http://127.0.0.1:<P>/api/v1/fiscal-years \
  -o /home/gcorbaz/devel/kesh-gate-logs/3211/amorcage/fiscal-years.json
curl -sS -H @/home/gcorbaz/devel/kesh-gate-logs/3211/amorcage/cle-amorcage.entete -H 'Content-Type: application/json' \
  -d '{"entryDate":"<A>-01-15","journal":"OD","description":"Apport en capital (amorçage 32-11)",
       "lines":[{"accountId":<id 1020>,"debit":"17345.10","credit":"0"},
                {"accountId":<id 2800>,"debit":"0","credit":"17345.10"}]}' \
  http://127.0.0.1:<P>/api/v1/journal-entries -o /home/gcorbaz/devel/kesh-gate-logs/3211/amorcage/E1.json
# P4 à P11 — une requête du tableau de l'AC-A2, envoyée une fois (R01 ; les autres de même, avec l'en-tête de version)
G=/home/gcorbaz/devel/kesh-gate-logs/3211 && curl -sS -X POST http://127.0.0.1:<P>/mcp \
  -H @$G/cle-parcours.entete -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' \
  --data-binary @$G/requetes/R01-initialize.json -D $G/reponses/R01.entetes -o $G/reponses/R01.json -w '%{http_code}\n'
```

(La forme des montants — `"17345.10"` — et des lignes se relit au code de `POST /journal-entries` à l'étape 0 ; la
table de P0 fait foi.)

- **L'agent ne lance pas `claude`** (C-32-11-28) et n'écrit rien dans `~/.claude.json`, `~/.claude/` ni un
  `.mcp.json` du dépôt ; il ne lance pas `socat` (la capture est un geste de Guy, AC-A8). Toute écriture va sous le
  worktree, sous `kesh-gate-logs/` (`docs/agents-orchestration.md` § 2.1) ou sous les dossiers neufs
  `/tmp/kesh-e2e-3211*` (§ 4), en chemin absolu. Pas même `claude --version` ni `claude --help`.
- **`curl` est l'outil du parcours**, déclaré au § *Gates* : aucune requête hors de P0, des tableaux R01-R09 et E01 à E04
  et de `GET /health`, aucune requête rejouée « pour voir » — une requête envoyée deux fois est un constat au journal.
- **Les gestes d'écran et leurs captures** se jouent par le navigateur piloté (AC-A1) ; aucune route ne les remplace.
- **La clé dans les journaux** : `cle-parcours.entete` porte le secret d'une clé de **démonstration**, révoquée en P11
  (et en fin d'AC-E2), base supprimée ; `amorcage/cle-amorcage.entete` celui de `amorcage-3211`, révoquée en P0 (d) ; il reste hors du dépôt, et aucune commande ne l'écrit au journal.
- **Le texte du manuel s'écrit depuis le journal**, non depuis cette fiche : l'ordre des gestes, les libellés d'écran,
  les messages exacts sont ceux que le parcours a rencontrés.

### Fichiers touchés (prévus) et modules

- `crates/kesh-api/tests/mcp_contrat.rs` (neuf ; aucune fixture, AC-B5),
  `crates/kesh-api/tests/audit_route_registry.rs` (AC-T1) — module **`kesh-api/tests`** ;
- `docs/api-external.md` — **docs** ;
- `docs/manual/fr/{admin-manual,user-manual,marketing-brochure}.{tex,pdf}` — **manuels FR** ;
- `README.md`, `website/roadmap.html`, `CHANGELOG.md` — **textes publics** ;
- `_bmad-output/…` (fiche, registre, `sprint-status.yaml`, prompts ; l'epic, annoté à la validation P1, R8, et à la P3, R2 : § 32-11 et § *Release v0.16.0*).

**Quatre modules** (≤ 5). Aucun code de production, aucune migration, aucun fichier du frontend ni de `kesh-i18n`.
Une inscription à une liste fermée de garde (AC-T2) touche `tests/textes_coherents.rs` ou `tests/mcp_gardes_lexicales.rs`
— même module.

### Le manuel (axe de revue)

- Chaque phrase de `sec:mcp-fonctionnement` et de `sec:agent-mcp` se relit **contre le code** au dernier état ; chaque
  geste de `sec:mcp-brancher` **contre le journal** (et le journal de T-8) ; la sortie de la revue cite les deux.
- `grep -nE 'sec:mcp[a-z-]*' docs/manual/fr/admin-manual.tex` : chaque `\ref` a son `\label`, une seule fois.
- Aucune seconde copie : ni de la matrice des portées (`sec:mcp-outils`), ni de la configuration du client
  (`sec:mcp-client`), ni de l'écran des envois (`sec:mcp-envois`).
- Aucun « Hermes » (`grep -n -i hermes docs/manual/fr/*.tex README.md website/*.html docs/api-external.md` → rien).

### Références

- Epic 32, § 32-11 (l. 516-541), § *Réponses de Guy* ; spike § 1.1, 1.3, 2.1, 2.4, 6 ; fiches 32-6 (l. 1259-1323), 32-7
  (AC-18 (8), AC-20, demandes 6 et 11 ; C-32-7-13, C-32-7-22), 32-8 (l. 1441-1443 ; C-32-8-26, C-32-8-27), 32-9
  (l. 1330-1349, Q2, demande 2), 32-10a (l. 1388-1394 ; C-32-10a-29), 32-10b (AC-19, AC-20) ; 31-4 (T8, C-31-4-15) ;
  rétrospective de l'Epic 31, § 3.2 et § 7.2.

## Points soumis à Guy

1. **Acceptes-tu qu'un agent lance `claude -p` sur ton compte, ce qui écrit dans `~/.claude/projects/` et
   `~/.claude.json` ?** Ce serait une **dérogation nommée** à `docs/agents-orchestration.md` § 2.1 (« Sans exception,
   pour tout agent : toute destination d'écriture est un chemin absolu sous le worktree assigné ou sous `$GATE_LOGS` »)
   et à la règle du `CLAUDE.md` (« un agent n'écrit jamais hors de son worktree ») ; `~/.claude.json` n'a pas de copie,
   et la session de l'orchestrateur l'écrit en même temps. Si oui, la variante est écrite en **Annexe** : l'agent y joue
   le parcours avec Claude Code lui-même, en mode non interactif, sur ton authentification et ton usage, et la fiche
   est amendée avant le développement (nouvelle passe de validation). **Le montage par défaut**, retenu sans ta
   réponse : l'agent joue le parcours **en client HTTP** (`curl`), sans `claude` (AC-A1, AC-A2) ; les gestes de Claude
   Code et la **capture** de ses requêtes réelles sont **ta recette** de la 0.16.0, depuis le PDF, vingt à trente
   minutes (AC-A8) ; le test de contrat de la CI ne dépend pas de cette capture (AC-B5). *Ma recommandation : **non** —
   le montage par défaut suffit* : il prouve le serveur, le manuel des écrans et la restauration sans toucher à tes
   fichiers ; ce qu'il ne prouve pas du client, ta recette le prouve, et elle est inscrite à la release de l'epic (§ *Release v0.16.0*).
2. **L'oracle du test de contrat : des clauses recopiées à la main, ou le schéma officiel ?** La fiche fait écrire
   chaque attendu depuis la phrase citée de la spécification `2025-11-25`, avec des listes closes de champs (AC-B2,
   AC-B4). L'alternative : versionner le `schema.json` officiel de la révision et valider chaque réponse avec une
   dépendance de test (`jsonschema`, absente du dépôt aujourd'hui, avec ses propres dépendances). *Ma recommandation :
   les clauses à la main pour la 0.16.0* — pas de dépendance, et chaque attendu se lit dans le test ; le schéma officiel
   se reprend le jour où Kesh réexamine le SDK (`rmcp`, Q8 de la spike). Le prix : une lecture fausse de la
   spécification passerait (§ *Ce que les gardes ne voient pas*).
3. **Jouer un libellé piégé avec le vrai client ?** (Q2 de la fiche 32-9.) **Ne se pose que si tu réponds oui au
   point 1** : observer ce que fait un agent réel devant une instruction cachée dans un libellé demande un agent réel,
   que le montage par défaut ne lance pas. Les pièges vivent derrière une fonctionnalité de compilation des seuls tests ;
   les jouer demande un binaire compilé avec elle, pour **un** appel de plus (Annexe). *Ma recommandation, si le point 1
   reçoit oui : oui, un seul appel, sur un binaire de parcours, sans toucher la démonstration publique* — le constat
   s'écrit, rien ne se corrige ici. Si le point 1 reçoit non, l'observation reste un angle mort écrit (§ *Ce que les
   gardes ne voient pas*).
4. **Si le parcours révèle un défaut, où le corrige-t-on ?** En particulier les textes de la poignée (`TEXTE_ENVOI_DEPOSE`,
   `instructions`) si l'agent ne relit pas ou relit en boucle (C-32-8-26 laissait « la 32-11 ajustera »). *Ma
   recommandation : jamais dans la 32-11* — elle observe, écrit et demande l'issue ; une correction de texte ou de code
   est une story courte à part, avant la release si le défaut empêche l'usage, sinon tracée. C'est la clôture qui évite le
   recyclage de la 31-4 ; elle reprend, pour les textes de la poignée, la promesse de C-32-8-26.
5. **Deux textes publiés disent qu'une comptabilité réelle est tenue dans Kesh depuis le 2026-10-23**, ce que ta
   précision du 2026-10-25 rend faux : la ligne v0.15.0 du `README.md` (l. 232 à `ed981c7d`) et le paragraphe
   d'introduction de la feuille de route du site, `website/roadmap.html:43` (« Kesh now keeps real books… since
   2026-10-23 its author's own accounts… are kept in Kesh »). La 32-11 ne touche que la ligne v0.16.0 de l'un et la
   ligne de l'E32 de l'autre. *Ma recommandation : les corriger tous deux à l'audit des documents de la release
   0.16.0*, comme le `CLAUDE.md` l'a été, sans réécrire le CHANGELOG publié.

## À recontrôler à l'étape 0 du développement

- **Fusions** : `git log --oneline origin/main` porte la 32-7, la 32-9a, la 32-9b, la 32-10a (ses parties) et la 32-10b ;
  sinon **arrêt**. `git log ed981c7d..origin/main --stat` : ce que chaque story a réellement livré.
- **Le serveur** : `mcp/mod.rs` (ordre des contrôles, `VERSIONS_SERVIES`, `MCP_CORPS_MAX`, `DELAI_OUTIL`) ;
  `repartiteur.rs` (méthodes servies, négociation d'`initialize`, codes `-32601`/`-32602`) ; `registre.rs` (outils et
  portées, au dernier état) ; `mcp/textes.rs` (`TEXTE_ENVOI_DEPOSE`, `instructions`) ; la réponse à un message de type
  réponse (B4) — chaque attendu de K1 à K22 relu contre le code **et** contre la clause.
- **La spécification `2025-11-25`** : les pages citées par la spike § 1.1, plus *lifecycle* (négociation de version) et
  les schémas de `InitializeResult`, `Tool`, `CallToolResult`, `Prompt`, `GetPromptResult` (clés **requises**, et
  ce qu'ils admettent de plus, types de bloc de `content`) ; le caractère facultatif d'`isError` et de la session
  (AC-B2) ; ce que la page *transports* dit d'une entrée que le serveur n'accepte pas (K21 : 202 ou un statut
  d'erreur, phrase entière) ; la page d'autorisation (K16) ; l'énumération de `role` (K14). Texte cité, date de
  consultation ; chaque assertion de K1 à K22 (K4 retiré) rangée selon la règle de l'AC-B2 — `DECISION_KESH` par
  défaut, `CLAUSE` sur une phrase citée en MUST seulement.
- **Claude Code** : rien à l'étape 0 — le montage par défaut ne le lance pas (C-32-11-28). La forme de la commande d'un
  prompt MCP que le manuel écrit (`/kesh-demo:<prompt> (MCP)`) vient de sa documentation (spike l. 253) et reste
  « non éprouvée » jusqu'à la recette de Guy (AC-A8). `curl --version` (l'option `-H @fichier`).
- **Le client HTTP** : le nom de l'argument de `kesh_lire_envoi` (`envoi`, `repartiteur.rs:73-77` à `ed981c7d`) ; les
  en-têtes que `transport.rs` exige (`Content-Type`, `Accept`, `MCP-Protocol-Version`) ; le contrôle de la clé avant le
  corps (le 401 de R08 dès `initialize`).
- **`proposer_prompt`** (AC-C1 (v), (vi), AC-D2, AC-D3) : relu au code fusionné de la 32-10a — sa portée
  (`prompts:write`), l'écriture d'une version « à valider », l'absence de `politique::autoriser` (fiche 32-10a, l. 203,
  821, 837, 854). Et `textes::INSTRUCTIONS` (`textes.rs:101` à `ed981c7d` : « Serveur MCP de Kesh, en lecture
  seule ») : s'il dit encore « en lecture seule » sans réserve au code fusionné, c'est un **constat**, issue demandée
  (AC-A7) — la 32-11 ne touche pas au code de production.
- **L'amorçage** (P0) : la forme du corps de `POST /api/v1/journal-entries` (montants en chaîne ou en nombre, ligne
  au crédit seul) au code de la route ; l'écran des clés d'API ; la forme des montants dans le texte de `kesh_bilan`
  (code de la 32-9) — d'où le motif de l'AC-A5 (b) ; l'argument `fiscalYearId` de `kesh_bilan`.
- **Les prompts** : qu'une version écrite par l'Administrateur est en vigueur sans approbation (P5 (b), fiche 32-10b
  AC-7 et AC-9, au code).
- **Les tests** : `tests/common/mcp.rs` (aides de routeur, de clé, de société, `MockConnectInfo`) ;
  `tests/common/manuel.rs` (`normaliser`) ; `audit_route_registry.rs` (partition, l. 877-929 à `ed981c7d`) ; les listes
  fermées de `textes_coherents.rs` (G18, G19) et de `mcp_gardes_lexicales.rs` qu'un fichier neuf peut toucher ; le test
  de la matrice de la 32-9 (ancre `% matrice-mcp`). **L'amorçage des prompts standard dans le routeur de test** (K14,
  M20) : la 32-10a les amorce au démarrage (`amorcer_standards`, sa fiche, l. 23 et AC-i10, l. 371), et le routeur de `tests/common/mcp.rs` ne passe pas par `main.rs` — `mcp_contrat.rs` l'appelle donc lui-même,
  ou réemploie l'aide que la 32-10a a posée pour ses tests ; relu au code fusionné, ligne citée.
- **Le manuel d'administration** : la structure de `sec:mcp` posée par la 32-7 (étiquettes `sec:mcp`, `sec:mcp-client`,
  `sec:mcp-erreurs`, `sec:mcp-outils`, `sec:mcp-envois`, `sec:mcp-prompts`) ; `sec:mcp-client` — « non éprouvée »,
  **Hermes** (fait 10) ; le compte des routes (fait 8) et sa ligne ; `:2926` ; `:2963` ; `:2250` ; ce que le manuel
  dit déjà des points de #765, #768, #773 (AC-C6 ; à `ed981c7d` : `:2238`, `:2949`).
- **Le manuel utilisateur** : `:3029-3032`, `:3153-3157`, `:3208`, `:3218` ; la place entre *Traçabilité* et *FAQ* ; ce
  que la 32-7, la 32-9 ou la 32-10b y ont ajouté.
- **Les textes publics** : `README.md` (la ligne v0.16.0, recomptée), `website/roadmap.html` (la ligne de l'E32,
  recomptée), `docs/api-external.md` (§ 6.4, § 9, `:35`), `marketing-brochure.tex` (`:390-400`, `:480`) — l'une des
  stories précédentes les a-t-elle touchés ?
- **Les issues** : état de #759, #761, #765, #768, #770, #773, #776 (`gh issue view <n> --json state`) ; une issue fermée
  sort de l'AC-C6.
- **Les prompts standard** : leurs noms (`kesh-assistant`, `kesh-developpement` à la fiche 32-10a, à relire au code) et
  l'écran de la 32-10b (création d'une version).
- **Les réponses de Guy** aux points 1 à 5 ci-dessus, et à la Q2 de la fiche 32-9 (point 3). Un « oui » au point 1
  **arrête** l'étape 0 : la fiche est d'abord amendée selon l'Annexe et revalidée.

- **LOW de la validation P5** : B1 — l'AC-A8 (l. ≈332) et l'AC-B5 (l. ≈426) écrivent encore « clauses de K1 à K3, K5 à K7 et K14 » ; K5 est désormais `DECISION_KESH` (tableau, AC-B2) : retirer K5 de ces deux énumérations à l'étape 0, ligne lue citée.

## Annexe — la variante soumise à Guy (point 1) : l'agent pilote `claude -p`

*Non retenue par défaut. Esquisse, non spécification : si Guy répond oui au point 1, la fiche est amendée à partir
d'elle et repasse en validation **avant** le développement ; tant qu'il n'a pas répondu oui, aucun de ces gestes ne
se joue (AC-A2).*

- **La dérogation**, en toutes lettres : à `docs/agents-orchestration.md` § 2.1 et au `CLAUDE.md` (« un agent n'écrit
  jamais hors de son worktree »), **limitée** aux écritures que `claude -p` fait de lui-même dans
  `~/.claude/projects/<dossier de lancement>/` et `~/.claude.json` ; accordée par la réponse de Guy, reprise mot pour
  mot dans le prompt du développement — sans elle, l'agent s'arrête. Les écritures concurrentes de `~/.claude.json`
  avec la session de l'orchestrateur sont un risque déclaré ; une copie préalable du fichier est à décider avec Guy.
- **Le montage du client** (celui de la fiche à la validation P1, C-32-11-26) : un dossier de lancement neuf hors de
  tout dépôt (`kesh-gate-logs/3211/cwd/`) ; un fichier MCP jetable passé par `--mcp-config` avec
  `--strict-mcp-config` ; `--allowedTools "mcp__kesh-demo"` et `--disallowedTools` des outils d'écriture ; un plafond
  de tours et de budget, arrêt sans plafond ; tout se constate à `claude --help`.
- **Les sessions, bornées et comptées** : chaque invocation de `claude -p`, `--resume` compris, est une session nommée
  (`session-<geste>.jsonl`, sans réemploi d'un nom au rejeu) ; la session qui lit les outils et le prompt reçoit
  « n'appelle aucun outil », et tout envoi qu'elle déposerait est un constat, refusé ou laissé, sans changer l'envoi
  décidé en P7 ; le compte des sessions et des envois s'écrit à l'AC-A2 et au point 1, recompté.
- **La capture par l'agent** (`socat` sur un port réservé auprès de l'orchestrateur), d'où une fixture versionnée des
  requêtes de Claude Code (K4, avec son témoin MF1) ; la forme du prompt à essayer est `/kesh-demo:<prompt>` (spike
  l. 253) ; sur une clé révoquée, le 401 arrive à `initialize`, non à l'appel d'un outil.
- **P-pièges** (point 3) : un appel, sur un binaire compilé avec la fonctionnalité `pieges`, base et instance nommées.
- **Les défauts relevés par la validation P2 sur ce montage** (F1 = R2, F3 = R6, F4 = R3, F7, F8, R7 :
  `kesh-gate-logs/3211-validate-p2-{R,F}.md`) sont à reprendre à sa réécriture ; ils ne valent pas pour le montage par
  défaut, où ils tombent.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-27 | création | Fiche rédigée en autonomie (Opus 5.5) contre `ed981c7d`, après lecture de l'epic (§ 32-11 et notes datées), de la spike (§ 1, 2, 6), des fiches 32-6, 32-6c, 32-8 (dans ce worktree) et des fiches 32-7, 32-9, 32-10a, 32-10b (dans leurs worktrees, en lecture seule), de la 31-4 et de la rétrospective de l'Epic 31, et des issues #759, #761, #765, #768, #770, #773, #776 (lecture). Constats au relevé : aucun test ne porte le contrat du protocole (fait 3) ; Kesh ne journalise pas la révision demandée (fait 6) ; le compte des routes du manuel est faux de deux (fait 8). Registre C-32-11-1 à C-32-11-24 ; `sprint-status.yaml` : `32-11-brancher-un-agent: ready-for-dev` ; prompt de validation P1 `3211-validate-prompt-p1.md` (Sonnet, lentilles R et F). Décomptes (recomptés sur la fiche par `grep`) : **33 AC** (A 8, B 6, C 9, D 5, E 3, T 2), **11 tâches** (T-0 à T-10), **16 mutations** M1 à M16 plus MT1, MT2, MF1, **22 cas** de contrat (K1 à K22), **11 gestes** de parcours (P1 à P11) plus P-pièges conditionnel, **15 faits**, **5 points** pour Guy, **4 modules**. |
| 2026-10-27 | validation P1 | Lentilles R et F (Sonnet, contexte frais) ; rapports `kesh-gate-logs/3211-validate-p1-{R,F}.md`. **Trend : 0 CRITICAL / 1 HIGH / 5 MEDIUM / 16 LOW** (R 0/1/1/7, F 0/0/4/9). Remédiation par l'orchestrateur de la boucle (agent de spécification, Opus 5.5), chaque prémisse relue au dépôt. **R1 (HIGH)** — vrai (`grep -nE 'journal_entries\|contacts\|invoices' crates/kesh-seed/src/lib.rs` → rien) : fait 16 ; geste **P0** (amorçage : clé d'API `read-write` créée à l'écran, trois `POST /api/v1/journal-entries` aux montants fixés, clé révoquée, bilan constaté à l'écran ; route `lib.rs:486`, ouverte aux clés `read-write`, `api-external.md:218`, `:265`, `:269`) ; AC-A5 (b) cherche 20 432.05 et 17 345.10 avec un **témoin positif** (le même `grep` sur `socat.log` après P7) ; `socat` lancé après P0 ; C-32-11-25. **F1** — vrai (l'accusé ne porte aucune donnée, `politique.rs` ; `kesh_bilan` prend `fiscalYearId`, fiche 32-9 l. 356) : la consigne de P6 est écrite et donne l'identifiant lu en P0 ; **un** envoi au parcours, **un** à la restauration ; sessions S1 à S4 comptées. **F2** — montage du client à P3 : dossier de lancement neuf hors dépôt (`kesh-gate-logs` n'est dans aucun dépôt, aucun `CLAUDE.md` parent, vérifié par `git rev-parse` et `ls`), `--strict-mcp-config`, aucun dossier de configuration jetable (l'authentification de Guy reste, ses sessions s'écrivent dans son magasin : déclaré, et « ne jamais écrire dans `~/.claude*` » retiré au profit de « jamais lui-même »), `--max-turns 8` et un budget si offert, arrêt sans plafond ; `claude mcp add` ne s'éprouve plus (la condition de l'AC-A8 retirée) ; C-32-11-5 amendé, C-32-11-26 ; Q1 dit le coût. **F3** — la liste de l'AC-A2 est la seule source : P3 est le montage, P5 en trois sous-gestes, P3-direct nommé, la variante « route de décision » des Dev Notes retirée (un geste d'écran que l'agent ne peut jouer l'arrête). **Réfuté en partie** : « créer une version personnalisée et l'approuver » — une version écrite par l'Administrateur est en vigueur sans approbation, *Approuver* ne portant que sur une version proposée par un agent (fiche 32-10b, AC-7 l. 328-343, AC-9 l. 384-395) : P5 (b) constate la version en vigueur, et se réécrit « l'approuver » si le code fusionné l'exige (étape 0). **F4, R2, R3** — l'oracle : `CLAUSE` (clés requises, types, valeurs imposées) et `DECISION_KESH` (listes closes K1, K7, K14 ; `isError` K8 et M10 ; session AC-B3 et M5 ; K10 ; K12 ; K17 ; K19 ; K22) par assertion ; attendus en littéraux ; aides `attendu_*` (`common/mcp.rs:626-663`, vérifié : elles appellent `mcp::textes::texte_envoi_*`) interdites au contrat, prouvé par les mutations **M17** et **M18** (sans garde de texte nouvelle, que l'en-tête de la fiche exclut) ; le schéma officiel se constate à l'étape 0 (prémisses des lentilles sur `isError` facultatif et les propriétés supplémentaires **non vérifiées** ici : aucun accès réseau) ; C-32-11-7 et C-32-11-8 amendés, C-32-11-27. **LOW intégrés** : M11 (K12 reste vert, même branche) ; M16 réécrite (`autoriser` rend `Envoi` pour `Required`, `politique.rs:106-111`, vérifié) ; M5 touche `reponse::rendre` ; `enveloppe_conforme(&Rendu, Option<i64>)` (F6) ; K21 à 400 avec sa clause à citer, sinon retiré et **angle mort** écrit (R4, F7 — la phrase « MUST return an HTTP error status code » vient de la mémoire des deux lentilles, non vérifiée) ; MF1 et l'en-tête de provenance (heure, longueur de la ligne `socat`), la fabrication dite angle mort (F8) ; `configuration_transmise.rs` (`MANUEL_ADMIN`, l. 2619) au gate ciblé, et les gardes `textes_coherents.rs:411`, `:579` à l'AC-T2 (F9) ; `KESH_TEST_MODE` retiré du montage (`lib.rs:1309-1314`, `main.rs:279`) et le build du frontend au montage (F10) ; ce qui part avec un retrait de l'AC-T1 (F11 ; l'AC-T1 reste) ; K10 par la route de décision (F12, `lib.rs:380`, `common/mcp.rs:666`, `:685`) ; `website/roadmap.html:43` au point 5 avec `README.md:232`, et l'AC-D3 qui lit 43, 223, 372, 375 (R5, F13 ; `grep -n -i mcp website/roadmap.html` → 43, 223, 372, 375) ; AC-C9 sous `LC_ALL=fr_FR.UTF-8`, `L.{1,3}agent`, règle de la césure de la 32-7 (R6) ; AC-C6 (R7) : **réfuté en partie** — #765 (5) est **déjà dit en entier** à `admin-manual.tex:2238` (« une sauvegarde prise entre-temps le porte encore »), non à moitié, et #773 (9) à `:2949` ; restent #768 (1) et #773 (8) ; l'epic annoté, l. 522 (R8 ; texte d'origine laissé) ; fait 2 (`registre.rs:46`) et fait 3 (littéraux de `initialize_ping_prompts`) (R9). **Écarts des lentilles** : la lentille R a lancé `claude --version` (sortie `2.1.311 (Claude Code)`), un `awk` sans programme redirigé vers `/dev/null` et une substitution de processus, toutes interdites par le prompt et sans effet sur le dépôt ; la version est **à reconfirmer à l'étape 0** (fait 14). La lentille F n'en déclare aucun. **Décomptes** (recomptés sur la fiche et le registre par `grep -c`) : **33 AC** (A 8, B 6, C 9, D 5, E 3, T 2), **11 tâches**, **18 mutations** M1 à M18 plus MT1, MT2, MF1, **22 cas** K1 à K22, **12 gestes** P0 à P11 plus deux conditionnels (P3-direct, P-pièges), **16 faits**, **5 points** pour Guy, **27 choix** (C-32-11-25 à 27 neufs ; C-32-11-2, 5, 7, 8 et 23 amendés par une ligne datée), **4 modules**. Symptômes grepés après patch : `ce que l'agent choisit`, `Ne jamais`, `.claude*`, `M1 à M16`, `onze`, `route de décision`, `déplace le dossier` → aucun résidu hors du Change Log (l'entrée de création, laissée telle quelle, garde « M1 à M16 ») ; « route de décision » reste à K10, où elle est le moyen prescrit (F12), non une variante de geste. Prompt de la P2 : `3211-validate-prompt-p2.md` (Opus, lentilles R et F). |
| 2026-10-27 | validation P2 | Lentilles R et F (Opus 5.5, contexte frais, prompt `3211-validate-prompt-p2.md`) ; rapports `kesh-gate-logs/3211-validate-p2-R.md` (0/0/6/5) et `-F.md` (0/0/3/8). **Trend : P1 0/1/5/16 → P2 0/0/9/13** (somme des deux lentilles ; doublons F1 = R2, F3 = R6, F4 = R3, F2 et F5 ≈ R5, comptés une fois par lentille). ⚠️ **Signal D5 de `CLAUDE.md` (§ *Règle de splitting préventif*, amendement du 2026-10-06), déclaré au Project Lead** : la sévérité n'a pas baissé au-dessus de LOW (MEDIUM 5 → 9), et la plupart des MEDIUM **naissent de la remédiation P1** — F1 = R2 (session S1 non bornée), F3 = R6 (écritures de `claude` dans `~/.claude*`), F4 = R3 (sessions mal comptées), F2 et F5 = R5 (étiquettes `CLAUSE` posées par la P1) — c'est-à-dire du parcours joué par `claude -p` et de l'oracle réétiqueté : un **recyclage**, non des défauts d'origine neufs. **Décision de l'orchestrateur, appliquée** : le pilotage de `claude` par l'agent de développement est retiré du montage par défaut et devient une **variante soumise à Guy** (point 1 réécrit en question, recommandation « non » ; esquisse en **Annexe**) ; l'agent joue P0 à P11 et la restauration **en client HTTP** par `curl` (C-32-11-28 : un test Rust est écarté, le parcours entrelaçant ses requêtes avec des gestes d'écran sur la même instance ; K10 joue déjà la même séquence sans écran, en CI) — neuf requêtes au parcours (R01 à R09), quatre à la restauration (E01 à E04), neuf au rejeu, écrites avant l'envoi et envoyées une fois, un envoi par instance **par construction** ; aucun `socat`, aucun `claude` ; la capture des requêtes réelles de Claude Code et ses gestes interactifs sont **la recette de Guy** (AC-A8, C-32-11-29), sans effet sur le contrat de la CI (AC-B5 : aucune fixture, **K4 et MF1 retirés**, numéro K4 gardé). **Retirés, parce qu'ils tombent avec le nouveau montage** : F1 = R2, F4 = R3, F8 (le rejeu ne joue plus de geste de Claude Code ; il est borné à un passage, P0 et requêtes depuis la fiche, gestes d'écran depuis le PDF) ; **F3 = R6 déplacé au point 1** (la dérogation à `docs/agents-orchestration.md` § 2.1 y est nommée, et l'Annexe la reprend) ; R7 et F7 (forme du prompt, 401 à `initialize`) repris à l'Annexe et, pour le manuel, à l'AC-A8 et à l'AC-C7. **R1 (MEDIUM, d'origine)** — vrai (fiche 32-10a, l. 203, 821, 837, 1392 relues : `proposer_prompt` écrit, hors `politique::autoriser`) : AC-C1 (v) et (vi) sans garantie générale de « lecture seule » — les outils de lecture sur une `Lecture` `READ ONLY`, seul `proposer_prompt` écrit, sa trace est la validation de la version par l'Administrateur ; fait 9 complété ; AC-D2 (« en lecture seule des livres », `README.md:233`) ; AC-D3 prend en charge `roadmap.html:375` (« read-only on the books ») et la formule « a built-in, read-only MCP server » de **`:223` et `:43`** (`sed -n 43p` : la même formule y est — relevé à cette remédiation), le reste de `:43` laissé à la release (point 5) ; § *À recontrôler* : `textes::INSTRUCTIONS` (`textes.rs:101` : « Serveur MCP de Kesh, en lecture seule ») — un constat, issue demandée s'il reste tel, la 32-11 ne touchant pas la production ; C-32-11-15 amendé. **R4 (MEDIUM, d'origine)** — vrai (sortie de la lentille) : AC-C9 reprend la règle de la 32-7 (AC-21, l. 887-889 de sa fiche, relues) — 0 sur le PDF de la base, 1 au PDF régénéré, les deux sorties citées à l'étape 0, un motif déjà présent remplacé ; `Outil du serveur MCP appelé` retiré des motifs ; le compte « N des M » déclaré « déjà vrai » si la 32-10b l'a écrit ; `grep -c` sur ligne aplatie = présence. **R5 = F2, F5 (MEDIUM, nés de la P1)** — vrais (spike l. 85-96 relues : 403 MUST et corps « permis », 400 MUST pour la version, 405 du `GET`, SHOULD pour l'en-tête absent ; l. 161-163 : une version servie) : règle de rangement écrite à l'AC-B2 (`CLAUSE` pour ce que la spike cite en MUST et les exigences de base nommées, chacune à citer ; **tout le reste sous `DECISION_KESH`**, « à constater à l'étape 0 sur la page », promu seulement sur une phrase en MUST) ; K3, K8 à K11, K14 à K18, K20, K21 rangés (C-32-11-27 amendé) — K11 (`-32602`) passé aussi sous `DECISION_KESH`, la page ne le donnant qu'en exemple, ce qu'aucune lentille n'avait relevé ; K21 n'a plus de branche « retiré » (F2 : inatteignable), l'angle mort correspondant est retiré. **LOW intégrés** : F6 (K14 exige une liste non vide portant le standard ; l'amorçage des standards dans le routeur de test au § *À recontrôler*, fiche 32-10a l. 23 et l. 371 — la l. 439 citée par la lentille n'a pas été retrouvée) ; F7 (le 401 dès la première requête, R08) ; F9 (deux ports `P` et `P_R` demandés ; dossiers `/tmp/kesh-e2e-3211-r/…` de la seconde instance ; P-pièges à l'Annexe) ; F10 (AC-B3 : enveloppe sur chaque réponse JSON, en-tête sur chaque `Rendu` ; la capture « en P3 » tombe avec la fixture) ; F11 = R9 (**quatre** mutations ajoutées, au plafond : M19 K13, M20 K14, M21 `INSTRUCTIONS` → K1, M22 texte du 400 → K17, `textes.rs:101` et `:55` relus ; les cas restants sans mutation écrits en angle mort ; le contrôle `grep` étendu à `\btextes\b`) ; R7 (forme `/kesh-demo:<prompt> (MCP)`, spike l. 253, à l'AC-A8 et à l'AC-C7) ; R8 (fait 13 : `:2941` vise le refus d'une décision sur un envoi changé, #768 (1) non dit) ; R10 (fait 5 : `:22`, `:26-28` ; fait 3 : `mcp_traduction.rs`, sept fichiers ; fait 2 : `kesh_balance` ; origine : `:1330-1333` — **réfuté en partie** : à `c5a063da` du worktree `kesh-32-9`, `:1378` **est** la demande 2, `grep -n '^[12]\. \*\*32-'` → 1377 et 1378) ; R11 (C-32-11-2, 4 et 22 amendés « P0 à P11 » ; `sprint-status.yaml` laissé, relevé daté). **Écart de la lentille R** : une commande a porté une redirection interdite (`2>&1`), sans fichier écrit ; la lentille F n'en déclare aucun. **Symptômes grepés** (valeurs) sur la fiche hors du Change Log : `claude -p` (le chapeau de la partie A, le point 1, l'Annexe), `S1`/`S4` → rien, `--resume` (l'Annexe), `~/.claude` (Dev Notes, point 1, Annexe), `socat` (fait 14, AC-A1 et Gates qui l'excluent, la capture de Guy à l'AC-A8, l'Annexe), `P3-direct` → rien, `P-pièges` (l'Annexe), `fixture` (fait 15, AC-B5, T-4, M, § *Ce que les gardes ne voient pas*, Fichiers, Annexe), `MF1` et `K4` (sites qui les retirent, Annexe), `Q1` → rien, `P+1`/`P+2` → rien, `transcript` → rien, `CLAUSE` (chaque site relu contre la règle de l'AC-B2) ; sur le registre, les entrées d'origine gardent leurs termes (relevés datés), les amendements P2 les annulent. **Aucune entrée passée réécrite** (Change Log : une ligne ajoutée ; registre : ajouts seuls, `git diff` sans ligne retirée). **Décomptes** recomptés à la source (`grep -c`) : **33 AC** (A 8, B 6, C 9, D 5, E 3, T 2), **11 tâches**, **22 mutations** M1 à M22 plus MT1, MT2 (MF1 retirée), **22 cas** numérotés K1 à K22 dont **21** actifs (K4 retiré), **12 gestes** P0 à P11, aucun conditionnel, **9 requêtes** au parcours, **16 faits**, **5 points** pour Guy, **29 choix** (C-32-11-28 et 29 neufs ; 1, 2, 4, 5, 6, 9, 15, 22, 24, 26, 27 amendés par une ligne datée), **4 modules**. **Passe 2 sur 8.** Prompt de la P3 : `3211-validate-prompt-p3.md` (**Sonnet**, lentilles R et F, contexte frais, axe des régressions de cette remédiation). |
| 2026-10-27 | validation P3 | Lentilles R et F (Sonnet 5.5, contexte frais, prompt `3211-validate-prompt-p3.md`) ; rapports `kesh-gate-logs/3211-validate-p3-R.md` (0/0/1/6) et `-F.md` (0/0/0/7). **Trend : P2 0/0/9/13 → P3 0/0/1/13** (somme des deux lentilles ; doublons F2 = R4, F1 = R6, F7 = R3, comptés une fois par lentille). Remédiation par l'agent de spécification, décisions de l'orchestrateur, minimale. **R1 (MEDIUM)** — vrai (AC-D4 : « joué sur une démonstration » alors que `claude` n'est pas lancé) : l'entrée du CHANGELOG décrit ce que la story livre (le serveur décrit au manuel, le parcours joué en client HTTP, le test de contrat) et dit : « la configuration de Claude Code est décrite au manuel, non éprouvée avant la recette ». **R2** — l'epic reçoit une ligne datée à la § *Release v0.16.0* (recette de Guy, AC-A8) et une annotation à la § 32-11 ; « déjà prévue » du point 1 devient « inscrite à la release ». **R3 = F7** — les Dev Notes nomment les dossiers `/tmp/kesh-e2e-3211*` (§ 4 de `docs/agents-orchestration.md`) ; le `.keshbackup` de P10 va sous `kesh-gate-logs/3211/export/` (AC-A4) ; la clé d'amorçage passe par un fichier d'en-tête (`amorcage/cle-amorcage.entete`), non en clair. **R4 = F2** — AC-A5 (c) vise le bloc texte de données de R07 (second bloc de `result.content`, `reponse.rs:60-70`) et nomme l'outil de décodage au journal. **F3** — R07 écrite après R06, une fois `<n>` connu. **F4** — l'interdit des Dev Notes couvre aussi P0 et `GET /health`. **F5** — fait 14 : plus de « à reconstater », la version se relève à la recette ; Dev Notes : pas même `claude --version` ni `--help`. **F6** — le rejeu T-8 écrit sous `kesh-gate-logs/3211/t8/` et ne réécrit rien du premier passage. **R5** — fait 13 : la commande est bornée (`sed -n 2928,2990p … | grep`). **R6 = F1** — AC-B2 : « cite en MUST » ne vise que K15 et K17 ; K5 et K20 sont à retrouver en MUST sur la page. **R7** — fait 6 renvoie à C-32-11-29 ; AC-A8 : le secret de la clé dans `capture-recette.log` se masque avant tout partage, la capture reste hors du dépôt. **Écarts des lentilles** : chacune a lancé une redirection interdite (R : vers `/dev/null` ; F : `2>&1 | head`), sans fichier écrit, comptes refaits sans redirection. **Symptômes grepés** (valeurs) hors Change Log : `joué sur une démonstration`, `reconstater`, `écrites avant`, `reponses-t8`, `déjà prévue` → aucun résidu ; `claude --version` (fait 14, Dev Notes qui l'interdisent) ; `C-32-11-6` → seulement les entrées historiques du registre. **Aucune entrée passée réécrite** (Change Log : une ligne ajoutée). **Décomptes inchangés**, recomptés par `grep -c` : **33 AC**, **11 tâches**, **22 mutations**, **22 cas K**, **12 gestes**, **9 requêtes**, **16 faits**, **5 points**, **29 choix**, **4 modules**. **Passe 3 sur 8.** Prompt de la P4 : `3211-validate-prompt-p4.md` (**Opus**, passe ciblée, une lentille *regression hunter* sur ce seul commit). |
| 2026-10-27 | validation P4 | Passe ciblée, une lentille *regression hunter* (Opus 5.5, contexte frais, prompt `3211-validate-prompt-p4.md`) sur le seul commit `fe66c4f8` ; rapport `kesh-gate-logs/3211-validate-p4.md` (0/0/2/6, aucun écart au prompt). **Trend : P3 0/0/1/13 → P4 0/0/2/6.** ⚠️ **Signal D5 (recyclage) déclaré : la remédiation P3 a recyclé** — les deux MEDIUM naissent d'elle (A1 de la phrase neuve du CHANGELOG, A2 de son propre Change Log). Remédiation par l'agent de spécification, décisions de l'orchestrateur, **en retirant plutôt qu'en corrigeant**. **A1 (MEDIUM) — retrait** : l'AC-D4 ne nomme plus Claude Code ; l'entrée décrit ce que la story livre (le fonctionnement du serveur MCP et le parcours d'administration joué sur une démonstration, la restauration comprise, le test de contrat en CI, le manuel utilisateur, `docs/api-external.md`) ; la phrase sur Claude Code s'écrit **à la release, par l'orchestrateur, après la recette de Guy** — ajouté à l'AC-A8 et, en annotation datée, à l'epic § *Release v0.16.0*. **A2 (MEDIUM) — annotation datée, entrée P3 non réécrite** : trois affirmations de l'entrée P3 étaient fausses au 2026-10-27. (a) « `joué sur une démonstration` … aucun résidu » : la valeur reste dans l'AC-D4, coupée par un retour à la ligne (l. 549-550 à `fe66c4f8`) — le grep avait cherché une phrase ; elle y est gardée volontairement (le parcours HTTP est joué sur une démonstration). (b) « `déjà prévue` … aucun résidu » : la valeur reste au registre (C-32-11-29, *Retenu*) ; une ligne datée l'amende, la recette étant inscrite à la § *Release v0.16.0* depuis la P3. (c) l'AC-D4 ne contenait ni « client HTTP » ni « test de contrat » : le premier n'y est toujours pas (le CHANGELOG ne décrit pas l'outil du parcours), le second y est désormais. **A3** — le fichier `amorcage/cle-amorcage.entete` est nommé en P0 (a), à l'AC-A4 et dans « La clé dans les journaux ». **A4** — T-1 reçoit `/tmp/kesh-e2e-3211-t8/…`, `kesh-gate-logs/3211/export/` et `t8/`. **A5** — fait 13 : la ligne rendue est préfixée `14:` (relative à la fenêtre), le calcul 2928 + 14 − 1 = 2941 est écrit. **A6** — fait 14 : la version de `claude` est demandée par l'AC-A8 (ajout) et l'AC-C2, à la recette. **A7** — K5 et K20 (`GET`) passent sous `DECISION_KESH` « à retrouver en MUST sur la page » dans le tableau et dans la règle de rangement de l'AC-B2. **A8** — AC-E2 : `<X_R>` se lit à l'écran, aucune requête de plus. **Écarts** : aucun relevé par la lentille ; l'agent de spécification a lancé, par mégarde, un `cat` redirigé vers `/dev/null` sans effet (aucun fichier écrit). **Symptômes grepés, mot à mot** (`Claude Code`, `éprouvée`, `prévue`, `démonstration`) : l'entrée de l'AC-D4 ne nomme plus Claude Code ; les autres occurrences de « éprouvée » (l. 123, 466, 556, 851, 872) concernent `sec:mcp-client` ou cette annotation, non le CHANGELOG ; « prévue » ne subsiste qu'à l'epic l. 537 (autre sujet) et au registre, amendé. **Aucune entrée passée réécrite** (Change Log : une ligne ajoutée ; registre et epic : une annotation datée chacun, aucune ligne retirée). **Décomptes inchangés**, recomptés par `grep -c` à la source : **33 AC** (A 8, B 6, C 9, D 5, E 3, T 2), **11 tâches**, **22 mutations**, **22 cas K**, **12 gestes**, **9 requêtes**, **16 faits**, **5 points**, **29 choix** ; **4 modules** relus, non recomptés par commande. **Passe 4 sur 8.** Prompt de la P5 : `3211-validate-prompt-p5.md` (**Sonnet**, passe ciblée, une lentille *regression hunter* sur ce seul commit). |
| 2026-10-27 | **validation P5 — VALIDATION TERMINÉE** | **Sonnet**, passe ciblée *regression hunter* sur `a3ba388f` (prompt `3211-validate-prompt-p5.md` ; rapport `kesh-gate-logs/3211-validate-p5.md`) : **0/0/0/1**. **Trend** : P1 (Sonnet ×2) 0/1/5/16 → P2 (Opus ×2) 0/0/9/13 → P3 (Sonnet ×2) 0/0/1/13 → P4 (Opus, ciblée) 0/0/2/6 → **P5 (Sonnet, ciblée) 0/0/0/1**. **Recyclage déclaré** en P2 (montage `claude -p`, retiré au profit du parcours en client HTTP) et en P4 (phrase du CHANGELOG, retirée : écrite à la release après la recette de Guy). **Clôture** sur une passe sans finding au-dessus de LOW, **5 passes sur 8**. Le LOW B1 va au § *À recontrôler à l'étape 0*. Écarts des lentilles (redirections sans écriture, `claude --version` en P1) déclarés à leurs entrées. Développement **après toutes les autres stories de l'Epic 32** ; points 1 et 3 soumis à Guy (variante `claude -p`, recommandation non). *(Écrit par l'orchestrateur.)* |
