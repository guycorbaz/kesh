# Story 32.11 : brancher un agent, de bout en bout

Status: ready-for-dev

*Fiche rédigée le 2026-10-27, en autonomie (Opus 5.5), worktree `kesh-32-11`, branche `story/32-11` partie de
`origin/main` `ed981c7d` (32-1 à 32-6c et 32-8 fusionnées ; 32-7, 32-9a, 32-9b, 32-10a et 32-10b non fusionnées, leurs
fiches lues en lecture seule dans leurs worktrees). Registre : `epic-32-choix-autonomes-3211.md` (C-32-11-1 à
C-32-11-27). Clé courte : `3211`. Issue de l'epic : #723 (CR « Serveur MCP intégré à Kesh »), en `refs` seulement. Les
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
un parcours **joué réellement** — activer le serveur, créer une clé, brancher Claude Code, appeler un outil, valider
l'envoi, relire la trace, restaurer une sauvegarde — décrit pas à pas au manuel et rejoué depuis le PDF, un **test de
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
  relecture ; C-32-8-26 et C-32-8-27), 32-9 (`32-9-outils-de-lecture.md:1330-1334`, `:1347-1349`, `:1378` : Q2 des
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
   `kesh_balance_agee`, `kesh_factures_fournisseur`, `kesh_facture_fournisseur`, `kesh_contacts`, `kesh_contact`,
   `kesh_pieces`, … — **liste à relire au registre à l'étape 0**, non à recopier d'ici).
3. **Les tests MCP existants** sont rangés par garde, non par clause du protocole : `tests/mcp_transport_e2e.rs` (2081
   lignes, dont `initialize_ping_prompts` l. 1027, `methodes_servies_liste_close` l. 983, `origine` l. 548,
   `enveloppe_http` l. 701, `notification_jamais_traitee` l. 948, `session_refusee` l. 1163), `mcp_reseau_e2e.rs`,
   `mcp_journal_e2e.rs`, `mcp_envois_e2e.rs`, `mcp_admin_routes_e2e.rs`, `mcp_gardes_lexicales.rs`. **Aucun fichier ne
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
5. **Les plages admises par défaut** comprennent la boucle locale (`kesh-db/src/entities/mcp_server_settings.rs:26-27` :
   `"127.0.0.0/8"`), filtre activé par défaut (`:21`) : un client de la station, ou un mandataire local, est admis sans
   réglage.
6. **Kesh ne journalise pas la révision que le client demande** : `repartiteur.rs:164` la lit
   (`params.get("protocolVersion")`) sans évènement ; seul `transport.rs:121` écrit un `debug!` quand
   `MCP-Protocol-Version` manque. L'observation (h) de la spike (quelle `protocolVersion` Claude Code demande) ne se lit
   donc **pas** au journal de Kesh : il faut voir l'échange (C-32-11-6).
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
     écriture par l'agent dans cette version » ; `website/roadmap.html:375`, mêmes deux affirmations en anglais.
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
    — #765 (5) entier). #773 (8) et #768 (1) n'y sont pas (`grep -n -F '200' … | grep -i envoi` et `grep -n -i recharg`
    → aucune phrase sur les envois).
14. **Claude Code sur la station** : `claude --version` → `2.1.311 (Claude Code)` (la spike lisait `2.1.310`, § 2.1,
    l. 209 ; même sortie à la validation P1, par une commande que le prompt de la lentille interdisait — **à reconstater
    à l'étape 0**). `socat` est installé (`/usr/bin/socat`). Le dépôt n'a **aucune** dépendance de schéma JSON
    (`grep -n '^name = "jsonschema"' Cargo.lock` → rien). **Rien de ce que la fiche dit du comportement de `claude`**
    (options, sessions, chargement du contexte, prompts en mode non interactif) **n'a été exécuté** : tout se constate à
    l'étape 0.
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

1. **AC-A1 — Le montage.** Une instance du **binaire du worktree** (non l'image publiée, qui n'existe pas avant la
   release — C-32-11-19), sur la recette du § *Montage E2E local* (`CLAUDE.md`) : base `kesh_3211` **vide**, port `P`
   attribué par l'orchestrateur, `RUST_LOG=info,kesh_api::mcp::reseau=debug` (pour la ligne d'admission du filtre,
   `sec:mcp-journal-filtre`), dossiers neufs `/tmp/kesh-e2e-3211/…`, `KESH_STATIC_DIR` sur le `frontend/build` **du
   worktree**, construit par `npm --prefix frontend run build` au montage (les écrans de P0 à P11 en dépendent).
   **Sans `KESH_TEST_MODE`** : il n'ajoute que `/api/v1/_test/*` (`kesh-api/src/lib.rs:1309-1314`) et un facteur de
   test (`main.rs:279`), dont aucun geste ne se sert, et le manuel décrit une installation sans lui ; les variables
   `KESH_SMTP_*` factices de la recette restent (aucun geste n'envoie de courriel). La démonstration se charge **par
   l'assistant** (écran), non par SQL ; elle ne porte aucune écriture (fait 16), d'où P0. Un mandataire de
   journalisation local relaie le client vers Kesh :
   `socat -v TCP-LISTEN:<P+1>,bind=127.0.0.1,reuseaddr,fork TCP:127.0.0.1:<P>` (C-32-11-6) ; sa sortie va à
   `socat.log`, **lancé après P0** (P0 parle à `P` directement : aucun montant de P0 n'entre dans `socat.log`). Les
   navigateurs Playwright de la recette E2E servent aux gestes d'écran et aux captures (script jetable sous
   `kesh-gate-logs/3211/`, jamais versionné) ; si l'agent ne peut pas piloter le navigateur, il **s'arrête** (AC-A7,
   dernier alinéa) — aucune route ne remplace un geste d'écran. Versions relevées : `GET /health` (champ `version`),
   `claude --version`, `socat -V | head -1`.
2. **AC-A2 — Le périmètre du parcours est clos** (C-32-11-2). Cette liste est **la seule source** des gestes : il
   compte **ces gestes, et aucun autre** — douze gestes, P0 à P11, et deux gestes conditionnels nommés, P3-direct et
   P-pièges. Aucune autre partie de la fiche (Dev Notes comprises) n'ouvre de geste ni de variante.
   - **P0** — *l'amorçage* (C-32-11-25), avant P1, sur `P` directement : (a) l'Administrateur crée à l'écran
     (`sec:api-keys`) une clé d'API `amorcage-3211`, portée `read-write` ; (b) il lit par cette clé l'identifiant de
     l'exercice de la démonstration (`GET /api/v1/fiscal-years`) et ceux des comptes `1020`, `2800`, `3200`, `6000`
     (`GET /api/v1/accounts`) ; (c) il saisit par `POST /api/v1/journal-entries` (fait 16) ces **trois écritures
     synthétiques**, dates dans l'exercice de la démonstration (année `<A>` = celle de l'exercice lu en (b)) :

     | n° | date | journal | libellé | débit | crédit | montant |
     |---|---|---|---|---|---|---|
     | E1 | `<A>-01-15` | `OD` | Apport en capital (amorçage 32-11) | `1020` | `2800` | 17 345.10 |
     | E2 | `<A>-02-10` | `Banque` | Prestation de démonstration | `1020` | `3200` | 4 321.55 |
     | E3 | `<A>-03-05` | `Banque` | Loyer de démonstration | `6000` | `1020` | 1 234.60 |

     Soldes attendus au bilan : `1020` Banque **20 432.05** (17 345.10 + 4 321.55 − 1 234.60), `2800` Capital social
     **17 345.10**, résultat de l'exercice 3 086.95 ; (d) il **révoque** `amorcage-3211` (une clé d'API donnée à un agent
     lit sans validation, `admin-manual.tex:2963`) ; (e) **témoin positif** : il lit le bilan à l'écran *Rapports* et y
     constate 20 432.05 et 17 345.10, capture à l'appui ; il note l'identifiant d'exercice `<X>` pour la consigne de P6.
     P0 se joue **depuis cette fiche**, au journal ; il n'entre pas au manuel comme geste (AC-C2).
   - **P1** — *Administration → Serveur MCP* : activer le serveur, plages par défaut (fait 5).
   - **P2** — créer la clé `parcours-3211`, portée **`books:read` seule**, validation **requise** (le défaut) ; le secret
     se lit une fois (capture, puis aucune copie hors du fichier de configuration jetable de P3).
   - **P3** — *le montage du client* (C-32-11-5, C-32-11-26), tout entier :
     - un **dossier de lancement neuf, hors de tout dépôt** — `kesh-gate-logs/3211/cwd/`, créé vide au geste ;
       `git -C <dossier> rev-parse --show-toplevel` doit échouer, et aucun `CLAUDE.md` dans ses parents (`ls`
       cité) : le `CLAUDE.md` de Kesh, ses skills et ses réglages de projet ne se chargent pas ; toutes les sessions
       du parcours partent de ce dossier (`--resume` retrouve une session par son dossier) ;
     - un **fichier MCP jetable** `kesh-gate-logs/3211/mcp-config.json` (forme de la spike § 2.4, l. 312-317) :
       serveur `kesh-demo`, `type: "http"`, `url: http://127.0.0.1:<P+1>/mcp`, en-tête `Authorization: Bearer <clé>`,
       passé par `--mcp-config` avec **`--strict-mcp-config`** (les serveurs MCP de Guy ne se chargent pas) ;
     - **aucun dossier de configuration jetable** : il retirerait l'authentification. Celle de Guy reste donc, et
       `claude` écrit ses sessions dans le magasin habituel de Guy (`~/.claude/projects/<dossier de lancement>/`) et
       met à jour `~/.claude.json` : c'est **accepté et déclaré** au journal (C-32-11-5 amendé) ; l'agent, lui,
       n'écrit rien dans ces fichiers. Restent chargés : `~/.claude/CLAUDE.md` de Guy, ses réglages, ses hooks, ses
       skills et greffons — la liste se relève dans le message `init` du transcript `stream-json` et se cite au journal ;
     - **outils** : `--allowedTools "mcp__kesh-demo"` **et** `--disallowedTools "Bash Edit Write"` (formes relues à
       `claude --help`) ; le message `init` doit ne lister que `kesh-demo` comme serveur MCP ;
     - **plafond de coût** : `--max-turns 8` par session (option de `claude -p`, à constater à `claude --help`), et un
       plafond de budget si `--help` en offre un ; **sans aucun plafond offert, l'agent s'arrête** et le dit (Q1).
     Tout ce qui touche au comportement de `claude` dans ce geste est **à constater à l'étape 0** (fait 14).
   - **P3-direct** *(conditionnel)* — si l'observation (b) montre un `Origin` envoyé, qui nommera `127.0.0.1:<P+1>` et
     recevra 403 (`Origin` étrangère) : le même montage, l'`url` sur `P` (sans capture), pour poursuivre ; le 403 est un
     constat (AC-A7). Sinon, absent.
   - **P4** — les outils de l'agent, lus à la **première session** (S1) : le message `init` du transcript et la réponse
     à `tools/list` de `socat.log` portent `kesh_version`, `kesh_lire_envoi` et les seuls outils `books:read` — aucun de
     `invoices:read`, `contacts:read`, `documents:read` (contrôlé contre la matrice de `sec:mcp-outils`). Aucune
     consigne propre : P4 se lit, il ne se demande pas.
   - **P5** — un **prompt standard** (`kesh-assistant` ou son nom réel, relu à l'étape 0), en trois sous-gestes :
     (a) l'Administrateur écrit une **version personnalisée** à l'écran des prompts (32-10b, *Enregistrer comme nouvelle
     version*), texte noté ; (b) il constate qu'elle est **en vigueur** (pastille *Personnalisé*) — une version écrite
     par l'Administrateur n'a pas d'approbation, *Approuver* ne portant que sur une version proposée par un agent (fiche
     32-10b, AC-7, l. 328-343, et AC-9, l. 384-395 ; à relire au code à l'étape 0, et P5b réécrit « l'approuver » si le
     code l'exige) ; (c) session S1 : `claude -p` reçoit pour seule consigne la commande du prompt (forme
     `/mcp__kesh-demo__<prompt>`, à constater à `claude --help` et à la page MCP de Claude Code à l'étape 0) ; l'agent
     reçoit le prompt (`prompts/get` à `socat.log`), dont le texte se compare à celui de (a) — observation (g) : sous
     quelle forme Claude Code le propose en mode non interactif ; s'il ne le sert pas, S1 ne sert qu'à P4, (c) est
     marqué « non éprouvé » (AC-A8) et la version en vigueur ne se constate qu'à l'écran.
   - **P6** — session S2 : l'agent demande **le bilan de l'exercice de la démonstration**, par cette consigne, écrite ici et passée
     telle quelle à `claude -p` : « Appelle l'outil `kesh_bilan` du serveur `kesh-demo` avec `fiscalYearId` = `<X>`,
     une seule fois. N'appelle aucun autre outil de données. Dis ce que tu as reçu. » (`<X>` lu en P0 (e) ; le nom de
     l'argument relu à la fiche 32-9, l. 356, et au code à l'étape 0). Sous la clé requise, l'appel rend « envoi
     n° `<n>` en attente » — **aucun montant ne part** (AC-A5 (b)). **Exactement un envoi** est déposé par P6 ; tout
     envoi de plus est un constat (AC-A7), noté au journal.
   - **P7** — *Administration → Envois du serveur MCP* : l'envoi `<n>` **modifié** puis envoyé — **une ligne changée**,
     qui ne porte ni 20 432.05 ni 17 345.10 (l'intitulé, par exemple), texte envoyé noté ; captures. La liste des envois
     n'en compte qu'un.
   - **P8** — session S3, `claude -p … --resume <S2>` : « L'envoi n° `<n>` a été validé ; relis-le par `kesh_lire_envoi`
     et recopie le texte reçu. » — observation (d) : combien d'appels à `kesh_lire_envoi` S2 a faits
     tant que l'envoi attendait (lus au transcript), et ce que l'agent fait de la relance.
   - **P9** — la trace : *Traçabilité* filtrée sur la période du parcours — les lignes « Outil du serveur MCP appelé »,
     les décisions d'envoi, la consultation du bilan (`report.generated`, #776) ; le journal du serveur porte la ligne
     d'admission du filtre (`DEBUG kesh_api::mcp::reseau`, client `127.0.0.1`).
   - **P10** — *Administration → Sauvegarde complète* : export du `.keshbackup` (pour E), **clé encore active**.
   - **P11** — révoquer la clé ; puis session S4, `claude -p … --resume <S2>` : « Appelle `kesh_version`. » — l'appel
     reçoit le 401 — observation (f) : Claude Code tente-t-il une découverte OAuth
     (`/.well-known/oauth-protected-resource`, visible à la capture `socat`) ?
   - **P-pièges** *(conditionnel, Q3)* — si Guy répond oui : un appel de plus, sur un binaire compilé avec la
     fonctionnalité `pieges` de `kesh-seed` (32-9, C-32-9-14), ce que fait l'agent devant un libellé piégé. Sinon, absent.

   **Les sessions et les envois, comptés** : quatre sessions `claude -p` au parcours (S1 : P4 et P5 (c) ; S2 : P6 ;
   S3 : P8 ; S4 : P11), une à la restauration (AC-E2), et une de plus si P-pièges se joue ; **un envoi** déposé et
   validé au parcours (P6, P7), **un** sur la seconde instance (AC-E2). Le rejeu T-8 (AC-A6) refait les quatre sessions
   et l'envoi.

   **Hors du parcours, décrits par le manuel tel que le code le fait, non rejoués** : une clé `dispensée` ; les portées
   `invoices:read`, `contacts:read`, `documents:read`, `prompts:write` et `proposer_prompt` ; le refus d'un envoi et son
   échéance ; le filtre d'adresse refusant ; un mandataire déclaré ; plusieurs sociétés ; le proxy de DSM et Traefik.
3. **AC-A3 — Le journal d'exécution**, `kesh-gate-logs/3211/parcours.log` : pour chaque geste P0 à P11, l'heure, la
   commande ou l'action d'écran, la sortie (ou la capture qui en tient lieu), et le constat ; en tête, les versions de
   l'AC-A1. Les observations de la spike § 2.4 **qui s'appliquent à Claude Code** y sont tranchées une à une, chacune
   « observé : … » ou « non observable dans ce mode : … » : (a) `initialize` puis `tools/list` sans session ; (b) un
   `Origin` envoyé ou non ; (c) un `progressToken` au `_meta` de `tools/call` ; (d) la relecture ; (e) le repli du runtime
   v2 (une requête de l'ère `2026-07-28` précède-t-elle `initialize` ?) ; (f) la découverte OAuth sur un 401 ; (g) la
   forme du prompt ; (h) la `protocolVersion` demandée et celle reçue.
4. **AC-A4 — Les pièces de preuve** (hors dépôt, `kesh-gate-logs/3211/`) : le transcript de chaque session Claude Code
   (`--output-format stream-json`, `session-S1.jsonl` à `session-S4.jsonl`), et le chemin de la copie que `claude`
   garde dans le magasin de Guy (P3), cité sans être recopié ; la sortie de `socat` (`socat.log`) ; les captures, thème clair,
   numérotées (`captures/NN-<geste>.png`) ; l'export CSV du journal d'audit de la période (*Traçabilité*). Aucune pièce ne
   contient de donnée réelle : la démonstration seule (`~/kesh-test/` n'est pas ouvert).
5. **AC-A5 — Ce que le parcours doit établir**, chaque point par une sortie citée au Dev Agent Record :
   (a) le client se branche et liste exactement les outils de P4 ; (b) **avant P7, aucune réponse de `/mcp` à un
   `tools/call` de données ne contient de montant du bilan** : les deux montants de P0, **20 432.05** et **17 345.10**,
   cherchés par `grep -cE` dans la part de `socat.log` antérieure à P7 → **0** ; le motif s'écrit après lecture du
   format des montants dans le texte de `kesh_bilan` (séparateur de milliers et décimal, code de la 32-9 relu à
   l'étape 0), et **un témoin positif** le valide : le **même** `grep`, sur la part de `socat.log` postérieure à P7 (la
   réponse de `kesh_lire_envoi` en P8, qui porte le texte validé), rend **au moins 1** pour chacun — sans quoi le
   contrôle passe à vide (format mal lu, ligne coupée par `socat -v`) et (b) **n'est pas établi** : constat au Dev Agent
   Record, et le motif se corrige avant de conclure. La capture de P0 (e) montre les mêmes montants à l'écran ;
   (c) **le texte que reçoit l'agent en P8 est, octet pour octet, celui validé en P7**, modification comprise (texte du transcript comparé au texte noté) ; (d) les lignes du journal d'audit
   de P9 existent, au nom du créateur de la clé, et la consultation du bilan s'y confond avec une consultation à l'écran
   tant que #776 est ouverte (constat, non défaut de la story) ; (e) après P11, le 401.
6. **AC-A6 — Le rejeu depuis le PDF** (T-8). Un second passage de P1 à P11, sur une démonstration neuve (base
   `kesh_3211_t8`) amorcée par P0 **joué depuis cette fiche** (P0 n'est pas au manuel), puis **en suivant le PDF** de
   *Brancher un agent* (AC-C2) et non le journal ; journal
   `kesh-gate-logs/3211/parcours-t8.log` ; toute différence entre le PDF et ce qu'il faut faire **corrige le manuel**
   (puis PDF régénéré). Un écart qui révélerait un défaut du produit suit l'AC-A7.
7. **AC-A7 — Un défaut révélé ne se corrige pas ici** (C-32-11-3). Si le parcours révèle un défaut du produit — un
   refus inattendu, une réponse que le client n'accepte pas, un texte de la poignée que l'agent ne suit pas (C-32-8-26) —
   la 32-11 **ne touche pas** au code de production : elle écrit le constat au journal et au Dev Agent Record, **demande
   l'issue** à l'orchestrateur, et le manuel décrit le comportement **réel** (ou se tait sur le geste). **Si le défaut
   empêche le parcours** (le client ne se branche pas, aucun outil ne s'appelle), l'agent **s'arrête** et le dit à
   l'orchestrateur, avec le constat : la correction est une story à part, avant la release (Q4).
8. **AC-A8 — Les gestes non éprouvés par l'agent** (Q1). Ce que l'agent ne peut pas jouer — `claude mcp add --scope
   local|user` (écrirait la configuration de Guy, C-32-11-5), la commande `/mcp` de la session interactive, l'invocation
   d'un prompt par une commande `/…` si le mode non interactif ne la sert pas — reste marqué **« non éprouvé »** au manuel,
   geste par geste, avec ce qu'il faut observer ; ces marques ne se retirent qu'après un rejeu par Guy, que
   l'orchestrateur reporte au Change Log. `claude mcp add` **ne s'éprouve pas** dans la 32-11, quelle que soit la
   documentation : un dossier de configuration jetable retirerait l'authentification de Guy (P3, C-32-11-26), et le
   geste écrirait sinon dans sa configuration.

### B — Le test de contrat

9. **AC-B1 — Un binaire neuf, `crates/kesh-api/tests/mcp_contrat.rs`** : `#[sqlx::test]`, routeur de production par
   `tests/common/mcp.rs` (pair simulé par `MockConnectInfo`, `oneshot`), **aucun réseau** ; il tourne en CI par le job
   backend (fait 4), ce que le rapport final demande à l'orchestrateur de constater au journal de la CI de la PR (nom du
   binaire et nombre de tests).
10. **AC-B2 — L'oracle est la spécification, jamais la sortie de Kesh** (C-32-11-7, C-32-11-27). Chaque cas porte
    l'une de deux marques, et chaque **assertion** se range sous l'une d'elles :
    - **`CLAUSE`** — l'URL de la page de la révision `2025-11-25` (date dans l'URL, comme la spike § 1.1), la section,
      et la **phrase citée** qui fonde l'attendu ; l'attendu s'écrit **depuis cette phrase** : ce que la spécification
      **exige** (clés requises et leurs types, valeurs imposées, forme générale de `CallToolResult` — `content[]` de
      blocs `type: "text"`).
    - **`DECISION_KESH`** — ce que la spécification laisse au serveur et que Kesh a décidé, avec sa référence
      (C-32-2-x, C-32-6-x, C-32-8-x) : la **liste close** des clés de `InitializeResult` (K1), de `Tool` (K7), de
      `Prompt` (K14) ; la **présence** d'`isError` (K8 ; la spécification le dit facultatif, à constater) ; **l'absence de
      session** (`Mcp-Session-Id`, AC-B3 ; la spécification dit « MAY assign a session ID », spike § 1.1) ; la forme de
      l'accusé et de la relecture (K10, 32-8 : `structuredContent {envoi, etat}`, second bloc de texte,
      `mcp/reponse.rs:44-75`) ; `-32602` pour un outil hors portée identique à l'inconnu (K12) ; 400 texte sur une
      version non servie, jamais `-32022` (K17) ; `readOnlyHint: true` (K7) ; 413 au-delà de 1 Mio (K22) ; lot refusé en
      400 (K19).
    Une assertion `DECISION_KESH` ne se présente jamais comme clause : le jour où une révision ajoute un champ, le test
    dit « décision de Kesh », non « écart au protocole ». Ce qui touche au **schéma officiel** (clés requises, caractère
    facultatif d'`isError`, session facultative) **se constate à l'étape 0, sur la page de la spécification**, phrase
    citée ; une affirmation de cette fiche que la page dément se réécrit selon la page.
    ⛔ **Les attendus sont des littéraux écrits dans le test** — textes d'accusé et de relecture, `INSTRUCTIONS`, codes,
    corps texte des refus — comme `initialize_ping_prompts` le fait déjà (fait 3). **Aucun attendu n'est recopié d'une
    réponse capturée de Kesh, ni construit par `kesh_api::mcp::textes`, ni par les aides `attendu_*` de
    `tests/common/mcp.rs`** (fait 3) : un contrat dont l'attendu sort du serveur testé est vert par construction
    (`CLAUDE.md` § *Inventorier les sites NON RÉSOLUS*, D4-ter). La preuve n'est pas une garde de texte (fiche, en-tête) :
    ce sont les mutations **M17** et **M18**, qui changent un mot des textes de production et doivent faire rougir K10 —
    elles passeraient à vide si l'attendu venait de `mcp::textes`. `grep -nE 'attendu_|mcp::textes'
    crates/kesh-api/tests/mcp_contrat.rs` → rien, sortie citée au Dev Agent Record (contrôle de revue, non garde).
11. **AC-B3 — L'enveloppe JSON-RPC, sur chaque réponse JSON du binaire** : une fonction
    `enveloppe_conforme(rendu: &Rendu, id_attendu: Option<i64>)` (`Rendu`, en-têtes comprises, `tests/common/mcp.rs:252`),
    appelée par chaque cas. `CLAUSE` : `"jsonrpc": "2.0"` ; `Some(id)` exige l'`id` **présent et égal** ; `None` n'est
    admis que pour une erreur de transport sans `id` (C-32-6-42 : K15, K16, K17 hors JSON, K19 à K22 selon leur corps),
    et exige alors l'`id` absent ou `null` selon la clause relue ; **exactement un** de `result` et `error` ;
    `error.code` entier, `error.message` chaîne. `DECISION_KESH` : **aucun en-tête `Mcp-Session-Id`** (serveur sans
    état, spike § 1.3).
12. **AC-B4 — Les cas, liste close** (C-32-11-8) — chacun avec ses assertions `CLAUSE` et `DECISION_KESH` :

    | # | requête | attendu |
    |---|---|---|
    | K1 | `initialize`, `protocolVersion: "2025-11-25"` | 200 JSON ; `CLAUSE` : les clés **requises** de `InitializeResult`, relues à l'étape 0, présentes et de bon type (`protocolVersion` = `"2025-11-25"`, `capabilities`, `serverInfo` avec `name` et `version`) ; `DECISION_KESH` : la liste **close** des clés, et `instructions` égal au **littéral** du test |
    | K2 | `initialize`, `"2025-06-18"` | `CLAUSE` : `protocolVersion` = `"2025-06-18"` |
    | K3 | `initialize`, `"2024-11-05"` | `CLAUSE` : `protocolVersion` = `"2025-11-25"` (négociation : le serveur répond une version qu'il sert) |
    | K4 | `initialize` **tel que Claude Code l'a envoyé en P3** (fixture, AC-B5) | 200 ; `CLAUSE` : `protocolVersion` selon la clause de négociation appliquée à la version demandée |
    | K5 | `notifications/initialized` | `CLAUSE` : 202, corps vide |
    | K6 | `ping` | `CLAUSE` : `result` = `{}` |
    | K7 | `tools/list` (clé `books:read`) | `CLAUSE` : chaque outil a `name` et `inputSchema` de `type` `"object"` ; `DECISION_KESH` : `annotations.readOnlyHint` = `true`, liste close des clés de `Tool`, aucun outil hors portée |
    | K8 | `tools/call` `kesh_version`, clé `dispensée` | `CLAUSE` : `result.content` de blocs `type` `"text"` ; `DECISION_KESH` : `isError` présent et faux |
    | K9 | `tools/call` d'un **outil de données** (32-9a), clé `dispensée` | idem K8 ; le texte porte la donnée posée par le test |
    | K10 | le même appel, clé **requise** | `CLAUSE` : forme de `CallToolResult` ; `DECISION_KESH` (32-8, AC-B5 et AC-B6) : l'accusé égal au **littéral** du test, aucune donnée ; puis l'envoi décidé **par la route de décision** (`POST /api/v1/admin/mcp/sends/{n}/decision`, avec `session` et `requete_api` de `tests/common/mcp.rs:666`, `:685` ; précédent `mcp_envois_e2e.rs:86`), `kesh_lire_envoi` rend le texte de relecture égal au **littéral** du test, et le second bloc égal à la chaîne que le test a envoyée à la décision |
    | K11 | `tools/call` d'un outil inconnu | `CLAUSE` : `error.code` = `-32602` |
    | K12 | `tools/call` d'un outil hors portée | `DECISION_KESH` (C-32-6-21) : **identique** à K11, corps pour corps |
    | K13 | méthode non servie (`resources/list`) | `CLAUSE` : `error.code` = `-32601` |
    | K14 | `prompts/list` puis `prompts/get` d'un standard (32-10a) | `CLAUSE` : `prompts[]` aux clés requises de `Prompt` ; `messages[]` : `role` ∈ {`user`, `assistant`}, `content.type` = `"text"` ; `DECISION_KESH` : liste close des clés de `Prompt` |
    | K15 | `Origin` étrangère | `CLAUSE` : 403 ; corps JSON-RPC sans `id` |
    | K16 | sans `Authorization` | 401 (`CLAUSE` de la page d'autorisation, relue) |
    | K17 | `MCP-Protocol-Version: 2026-07-28` | `DECISION_KESH` (C-32-2-1) : 400, corps texte égal au **littéral** du test, **pas** de `-32022` |
    | K18 | `MCP-Protocol-Version` absent | `CLAUSE` : admis (même réponse que K6) |
    | K19 | lot JSON-RPC (`[…]`) | `DECISION_KESH` : 400 |
    | K20 | `GET /mcp` ; `DELETE /mcp` | 405, `Allow: POST` (`CLAUSE` de la page *transports*, relue) |
    | K21 | message de **type réponse** (`result` sans `method`) | `CLAUSE` : 400 — un serveur qui ne peut accepter l'entrée « MUST return an HTTP error status code (e.g., 400) », phrase **à citer à l'étape 0** depuis la page *transports* (de mémoire des deux lentilles de la P1 ; la spike, § 1.1, n'en cite que la première moitié) ; si la page relue la dément, K21 est **retiré** du binaire, l'écart va au Dev Agent Record et à une issue demandée (AC-A7), et le cas devient un **angle mort** écrit au § *Ce que les gardes ne voient pas* — jamais `#[ignore]` |
    | K22 | corps de 1 Mio + 1 octet | `DECISION_KESH` : 413 |

    Les cas K1 à K22 **ne remplacent ni ne modifient** les tests existants (fait 3) : ils en recoupent une partie,
    rangée par clause (C-32-11-8).
13. **AC-B5 — La fixture de Claude Code** : `crates/kesh-api/tests/fixtures/mcp-contrat/` porte les **requêtes**
    (jamais les réponses) que Claude Code a envoyées en P3 et P4 (`initialize`, `notifications/initialized`,
    `tools/list`), prises de `socat.log`, avec un en-tête de provenance (client et version, date, commande, et **l'heure
    et la longueur** de la ligne `socat` d'origine, qu'une revue retrouve au journal du Dev Agent Record) ; la clé y est
    **remplacée** par un marqueur que le test substitue. Un **témoin** dans K4 exige que `clientInfo.name` de la fixture
    nomme Claude Code — une requête synthétique substituée **par négligence** fait rougir (MF1) ; le témoin attrape
    l'oubli, **non la fabrication** (§ *Ce que les gardes ne voient pas*). Si P3 n'a pas pu se jouer, K4 est **retiré** et
    l'écart déclaré (aucune fixture inventée).
14. **AC-B6 — Les mutations** (§ *Tests et mutations*) : M1 à M18, chacune tuée par le cas nommé.

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
    client, non imposé à l'agent ; (v) **la validation avant envoi**, par clé ; (vi) **ce que Kesh garantit et ce qu'il
    ne garantit pas** — la lecture seule est tenue **par l'application** (outils sur une `Lecture` à connexion
    `READ ONLY`), non par un utilisateur MariaDB limité (Q11 de la spike : les mots « par construction » ne s'écrivent
    pas) ; un appel **écrit** (dernière utilisation, journal) ; le filtre d'adresse n'est pas un pare-feu ; une clé
    `dispensée` envoie sans montrer ; une clé d'API donnée au même agent lit sans validation (`:2963`) ; l'enveloppe
    « donnée non fiable » **aide** l'agent à ne pas suivre un libellé, elle ne l'en empêche pas ; ce qui part chez le
    fournisseur du modèle n'est plus dans Kesh ; et les limites de l'AC-C6.
16. **AC-C2 — *Brancher un agent***, `\subsection{Brancher un agent}\label{sec:mcp-brancher}` sous `sec:mcp`, après
    `sec:mcp-client` : P1 à P11 **tels que joués**, commandes copiées du journal (chemins et clés en `<…>`) — P0 n'y
    est pas un geste, une phrase dit seulement que le parcours a été joué sur une démonstration où quelques écritures
    avaient été saisies —, chaque
    capture décrite en mots (aucune image au manuel) ; la configuration du client par **renvoi** à `sec:mcp-client`, sans
    seconde copie (C-32-7-22) ; ce que l'agent voit d'un envoi en attente et ce qu'on fait à l'écran ; les observations
    de l'AC-A3 qui changent un geste, **datées et rapportées à la version de Claude Code** (« observé avec Claude Code
    2.1.x, le … ») ; les gestes non éprouvés marqués (AC-A8) ; un paragraphe *Après une restauration* (AC-E3).
17. **AC-C3 — `sec:mcp-client` amendée en place** (C-32-7-22) : la mention « non éprouvée » se retire **des seuls
    gestes éprouvés** (AC-A8) ; ce que le parcours réfute se corrige ; la forme par fichier jetable (`--mcp-config`,
    `--strict-mcp-config`) s'y ajoute si elle a servi ; **aucune mention de Hermes** — si la 32-7 en a laissé une, elle
    se retire (C-32-11-18).
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
    ou le refuse ; une clé dispensée envoie sans montrer ; **invoquer un prompt** dans Claude Code, sous la forme observée
    en P5 (ou « non éprouvé », AC-A8) ; **reconnaître l'activité de l'agent** au journal (et #776) ; ce que Kesh garantit
    et ne garantit pas, en quatre ou cinq phrases, renvoyant au manuel d'administration pour le détail. **Aucune phrase
    ne promet une écriture** : l'agent ne modifie pas les livres ; il peut proposer un prompt, qu'un Administrateur
    approuve ou refuse.
22. **AC-C8 — La brochure** : `marketing-brochure.tex:480` perd « MCP server » (reste « Webhooks pour assistants IA
    conversationnels », ou la ligne se réécrit sans MCP) ; la liste livrée (`:390-400`) gagne une ligne « Serveur MCP
    intégré : un agent IA du réseau local **lit** les livres, désactivé par défaut, clés dédiées à portées fines,
    données montrées avant l'envoi sous une clé à validation requise. *(v0.16.0)* » — sans promesse d'écriture.
23. **AC-C9 — Les PDF.** `admin-manual.pdf`, `user-manual.pdf`, `marketing-brochure.pdf` régénérés (`make fr` dans
    `docs/manual/`, sous `flock …/.gate.lock`) et commités. Contrôle **aplati**, motif par motif, sans `-i`, commande et
    compte cités au Dev Agent Record, **sous une locale UTF-8 déclarée** : `LC_ALL=fr_FR.UTF-8 pdftotext
    docs/manual/fr/<m>.pdf - | tr '\n' ' ' | tr -s ' ' | grep -cE '<motif>'` — admin : `Comment fonctionne le serveur
    MCP`, `Brancher un agent`, `2025-11-25`, le compte « N des M » recompté, `#776` ; utilisateur : `L.{1,3}agent IA et
    le serveur MCP` (l'apostrophe typographique `’` occupe trois octets : sous `LC_ALL=C`, `L.agent` rend 0 — constat de
    la validation P1, R6), `#776`, `Outil du serveur MCP appelé` ; brochure : `Serveur MCP intégré`, et `MCP server` → 0.
    **Césure** (règle reprise de la 32-7, AC-21) : un motif que la césure coupe dans le PDF aplati (`2025-11- 25`) se
    raccourcit (`2025-11-`), et l'écart se déclare. Aucun `??` (référence non résolue) dans les trois PDF aplatis ;
    aucun caractère U+FFFD.

### D — Les textes publics

24. **AC-D1 — `docs/api-external.md` § 6.4** (C-32-11-14) : réécrit — Kesh a son **propre serveur MCP** depuis la
    0.16.0, sur `/mcp`, avec ses clés `kesh_mcp_` (renvoi au manuel d'administration, § *Le serveur MCP*) ; une clé
    d'API (PAT) y est refusée ; l'ancien conseil (serveur MCP générique avec une clé `read`) **reste possible** mais passe
    **sans** la validation avant envoi ni le journal des appels MCP, ce que le texte dit en une phrase ; l'exemple JSON à
    `baseUrl` `/api/v1` est retiré ou marqué comme celui de cette voie. **§ 9** : « serveur MCP Kesh-natif » sort de la
    phrase « Hors périmètre » (`:831`) ; rien d'autre du § 9 ne change.
25. **AC-D2 — `README.md`, ligne v0.16.0** (l. 233 à `ed981c7d`, recomptée) : les deux affirmations réécrites (C-32-11-15)
    — « les données montrées à l'Administrateur avant de partir à l'agent, sous une clé à validation requise (le défaut) ;
    une clé dispensée les envoie sans les montrer, chaque envoi journalisé » ; « l'agent n'écrit rien dans les livres ; il
    peut proposer un prompt, qu'un Administrateur approuve ou refuse ; ses propositions d'écritures sont prévues pour la
    0.17.0 ». Le statut reste **🚧 En cours** ; la section *Fonctionnalités* et le ✅ sont à la release (C-32-11-16).
26. **AC-D3 — `website/roadmap.html`** (l. 375, recomptée) : les mêmes deux corrections, en anglais ; « Planned, not yet
    released » reste. `grep -n -i mcp website/*.html`, sortie citée : à `ed981c7d`, `roadmap.html:43` (le paragraphe
    d'introduction, qui nomme le serveur et dit aussi « Kesh now keeps real books… since 2026-10-23 », faux depuis la
    précision de Guy du 2026-10-25), `:223`, `:372`, `:375` ; les lignes 43 et 223 sont **lues et laissées** à l'audit
    de release (point 5 pour Guy), la 32-11 ne touchant que la ligne de l'E32.
27. **AC-D4 — CHANGELOG**, sous `## [0.16.0] — Non publié`, `### Ajouté` : « **Brancher Claude Code sur Kesh** : le
    manuel d'administration décrit le fonctionnement du serveur MCP et le branchement de Claude Code, pas à pas, joué sur
    une démonstration ; le manuel utilisateur dit ce que l'agent peut lire et ce qu'on montre avant l'envoi ;
    `docs/api-external.md` renvoie au serveur intégré. » — sans date (la release la pose).
28. **AC-D5 — Aucun texte ne promet une écriture.** `grep -n -i -E 'MCP|agent' README.md website/*.html
    docs/api-external.md` et le PDF aplati des trois manuels relus : aucune phrase ne dit que l'agent écrit, modifie ou
    crée dans les livres (sortie citée, phrases relues).

### E — La restauration jouée

29. **AC-E1 — L'export** est celui de P10 (clé active, serveur activé, prompt personnalisé de P5, envois décidés).
30. **AC-E2 — La restauration** par le chemin de l'**installation neuve** (31-1b) : une seconde instance du binaire du
    worktree, port `P+2`, base `kesh_3211_r` **vide**, l'assistant → *Restaurer une sauvegarde* ; puis, constatés à
    l'écran et cités au journal `kesh-gate-logs/3211/restauration.log` : le serveur MCP **activé** (réglages restaurés
    tels quels, Q7 du kickoff) ; la clé `parcours-3211` listée active ; le prompt personnalisé **et son historique** ; les
    envois de P7 ; le bilan de P0 à l'écran *Rapports* (20 432.05, 17 345.10) ; puis Claude Code, configuré sur `P+2`
    **directement** (sans second `socat`), depuis un second dossier de lancement neuf `kesh-gate-logs/3211/cwd-r/`, avec
    **la même clé** et le montage de P3 : une session (`tools/list` lu au message `init`, la consigne de P6 avec
    l'identifiant d'exercice lu sur la seconde instance), **un** envoi déposé, validé tel quel à l'écran, relu par
    `--resume`. La révocation de P11 se joue **après** l'export, sur la première instance
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
- [ ] **T-1 — le montage** (AC-A1) : bases `kesh_3211`, `kesh_3211_r`, `kesh_3211_t8`, `kesh_e2e_3211` créées par
  `kesh`, migrations du worktree ; build du frontend du worktree ; ports attribués par l'orchestrateur ; `socat` ;
  versions relevées.
- [ ] **T-2 — le parcours P0 à P11** (AC-A2 à AC-A5, AC-A7, AC-A8), journal, transcripts, capture `socat`, captures
  d'écran ; P3-direct et P-pièges selon leur condition.
- [ ] **T-3 — la restauration** (AC-E1, AC-E2) ; journal.
- [ ] **T-4 — le test de contrat** (AC-B1 à AC-B5) : clauses relues à la spécification `2025-11-25` (pages citées par la
  spike § 1.1, et les pages *lifecycle*, *tools*, *prompts* de la même révision), fixture tirée de `socat.log`.
- [ ] **T-5 — la garde du compte** (AC-T1) et le compte du manuel (AC-C4).
- [ ] **T-6 — les manuels** (AC-C1 à AC-C3, AC-C5 à AC-C8, AC-E3), écrits **depuis le journal** ; PDF régénérés sous
  verrou (AC-C9).
- [ ] **T-7 — les textes publics** (AC-D1 à AC-D5).
- [ ] **T-8 — le rejeu depuis le PDF** (AC-A6) ; corrections du manuel ; PDF régénérés.
- [ ] **T-9 — mutations** (§ *Tests et mutations*), une à une, restauration par `git checkout --` + `touch`.
- [ ] **T-10 — gates** (§ *Gates*), Dev Agent Record, `sprint-status.yaml` → `review`, prompt de revue P1, et la liste
  des issues demandées (AC-A7).

## Tests et mutations prévus

Les mutations M1 à M18 portent sur le **code de production** de `mcp/` (au dernier état, relu à l'étape 0) ; chacune
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

Mutations de la garde du compte : **MT1** — le manuel dit un « N » de moins → AC-T1 rouge ; **MT2** — une route du
registre passe de `Traced` à `NoMatter` (sans toucher le manuel) → AC-T1 rouge (et `the_registry_partition…`). Mutation
du témoin : **MF1** — `clientInfo.name` de la fixture remplacé par `"synthetique"` → K4 rouge ; elle prouve que le
témoin attrape une substitution négligente, non une fixture fabriquée avec le bon nom (§ *Ce que les gardes ne voient
pas*).

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
- **PDF aplatis** : AC-C9, commandes et comptes cités.
- **Recette visuelle** : la 32-11 n'ajoute ni ne modifie de contrôle visible ; ses captures (AC-A4) sont des **preuves
  du parcours**, thème clair, hors dépôt, citées au Dev Agent Record et au prompt de revue.
- Revue de code : P1 complète (Sonnet ou Opus, jamais Haiku), lentilles B, E, A ; axes nommés : **le manuel (`.tex` et
  PDF aplati) contre le journal d'exécution et contre le code**, **l'oracle du contrat (aucun attendu tiré de Kesh)**,
  **aucune promesse d'écriture**, **aucune donnée réelle, aucun secret** dans un fichier versionné.

## Ce que les gardes ne voient pas

- **Le client en mode interactif.** Le parcours de l'agent se joue en mode non interactif (`claude -p`) : la commande
  `/mcp`, l'invocation d'un prompt par `/…`, et la conduite d'un agent qui attend un humain n'y sont pas ce qu'elles
  sont dans une session ouverte. Les gestes non joués restent marqués (AC-A8) ; Guy les rejoue (Q1).
- **Les versions futures de Claude Code.** La fixture fige les requêtes d'**une** version (2.1.311 au relevé) ; une
  version qui demande une autre révision, ou passe à `2026-07-28` sans repli, n'est vue que par un nouveau parcours. Le
  manuel date ses observations.
- **La révision `2026-07-28`**, non servie (spike § 1.3) : le contrat vérifie le refus (K17), non un repli du client.
- **L'oracle écrit à la main** : les clés requises de `InitializeResult`, `Tool`, `Prompt` sont recopiées de la
  spécification par l'agent ; une lecture fausse de la spécification passe (Q2). Le schéma officiel ne le serait pas.
  Les listes closes sont des décisions de Kesh (AC-B2) : elles rougissent sur un ajout que la spécification permet.
- **La fixture fabriquée** : le témoin de K4 (MF1) attrape la substitution négligente, non une requête écrite à la main
  avec le bon `clientInfo.name` ; seule la ligne `socat` d'origine, citée par l'en-tête de provenance et retrouvée au
  journal du Dev Agent Record, en prouve la provenance — une revue la lit, aucun test ne le peut.
- **K21, s'il est retiré** (AC-B4) : la réponse de Kesh à un message de type réponse n'est alors tenue par aucun cas du
  contrat ; l'angle mort s'écrit ici, avec l'issue demandée.
- **Le contexte de l'agent d'essai** : il porte `~/.claude/CLAUDE.md` de Guy, ses réglages, hooks, skills et greffons
  (P3) ; un agent « nu » chez un autre utilisateur peut se conduire autrement. Ce qui est chargé se cite au journal,
  non se neutralise.
- **DSM, Traefik, le NAS** (#770 (5), demande 11 de la 32-7) : le parcours se joue sur la boucle locale, sans
  mandataire. La vérification de `X-Forwarded-For` derrière le proxy de DSM se fait chez Guy, à la mise à jour du NAS ;
  la 32-11 n'en affirme rien.
- **L'image publiée** : le parcours éprouve le binaire du worktree ; l'image 0.16.0 n'existe qu'à la release, qui
  rejoue un parcours court (C-32-11-19).
- **Ce que l'agent fait d'une donnée reçue** : aucune garde ; la validation montre ce qui part, le journal le trace,
  le prompt de l'agent de développement l'interdit (32-10a) — une consigne, non une garde.
- **L'injection par libellé** : vue seulement si Q3 reçoit oui, et alors sur un seul appel.
- **Le manuel**, hors du compte des routes : aucun gate ne le lit ; le prompt de revue nomme l'axe et exige le PDF.
- **La donnée envoyée au fournisseur du modèle** pendant le parcours : la démonstration et les trois écritures
  synthétiques de P0 seules, ce que l'epic accepte (« données chez le fournisseur du modèle acceptées ») ; le parcours
  consomme l'usage du compte Claude de Guy, sous le plafond de P3 (Q1).

## Dev Notes

### Le parcours, en commandes (indicatif : la liste des gestes de l'AC-A2 fait foi ; les options se relisent par `claude --help` à l'étape 0)

```sh
# terminal A — Kesh (recette du § Montage E2E local, sans KESH_TEST_MODE, port P, base kesh_3211 vide)
RUST_LOG=info,kesh_api::mcp::reseau=debug … KESH_PORT=<P> KESH_STATIC_DIR=<worktree>/frontend/build \
  DATABASE_URL=…/kesh_3211 cargo run -p kesh-api
# P0 — sur P directement, avec la clé d'API d'amorçage (révoquée à la fin de P0)
curl -s -H "Authorization: Bearer <clé amorcage-3211>" http://127.0.0.1:<P>/api/v1/fiscal-years
curl -s -H "Authorization: Bearer <clé amorcage-3211>" -H 'Content-Type: application/json' \
  -d '{"entryDate":"<A>-01-15","journal":"OD","description":"Apport en capital (amorçage 32-11)",
       "lines":[{"accountId":<id 1020>,"debit":"17345.10","credit":"0"},
                {"accountId":<id 2800>,"debit":"0","credit":"17345.10"}]}' \
  http://127.0.0.1:<P>/api/v1/journal-entries          # puis E2 et E3, même forme
# terminal B — le mandataire de journalisation, lancé après P0
socat -v TCP-LISTEN:<P+1>,bind=127.0.0.1,reuseaddr,fork TCP:127.0.0.1:<P> 2>> /home/gcorbaz/devel/kesh-gate-logs/3211/socat.log
# l'agent — depuis le dossier de lancement neuf de P3, fichier MCP jetable, outils et tours bornés
cd /home/gcorbaz/devel/kesh-gate-logs/3211/cwd && claude -p "<consigne de P6>" \
  --mcp-config /home/gcorbaz/devel/kesh-gate-logs/3211/mcp-config.json --strict-mcp-config \
  --allowedTools "mcp__kesh-demo" --disallowedTools "Bash Edit Write" --max-turns 8 \
  --output-format stream-json --verbose > /home/gcorbaz/devel/kesh-gate-logs/3211/session-S2.jsonl
claude -p "<consigne de P8>" --resume <S2> …   # S3, après la décision de P7 ; S4 de même pour P11
```

(La forme des montants — `"17345.10"` — et des lignes se relit au code de `POST /journal-entries` à l'étape 0 ; la
table de P0 fait foi.)

- **La configuration de Guy n'est pas un banc** (C-32-11-5 amendé) : l'agent n'écrit **lui-même** ni `~/.claude.json`,
  ni `~/.claude/`, ni un `.mcp.json` du dépôt, et ne lance pas `claude mcp add` (AC-A8). Ce que `claude` écrit de
  lui-même dans le magasin de Guy (sessions, compteurs) est accepté et déclaré (P3), et ne se supprime pas ensuite.
- **`Origin` et le port du mandataire** : traité par le geste conditionnel **P3-direct** (AC-A2), seule variante admise.
- **Les gestes d'écran et leurs captures** se jouent par le navigateur piloté (AC-A1) ; aucune route ne les remplace.
- **La clé dans les journaux** : `socat.log` et `mcp-config.json` portent le secret d'une clé de **démonstration**,
  révoquée en P11 (et en fin d'AC-E2), base supprimée ; ils restent hors du dépôt. La fixture versionnée (AC-B5) porte
  un marqueur à sa place.
- **Le texte du manuel s'écrit depuis le journal**, non depuis cette fiche : l'ordre des gestes, les libellés d'écran,
  les messages exacts sont ceux que le parcours a rencontrés.

### Fichiers touchés (prévus) et modules

- `crates/kesh-api/tests/mcp_contrat.rs` (neuf), `crates/kesh-api/tests/fixtures/mcp-contrat/` (neuf),
  `crates/kesh-api/tests/audit_route_registry.rs` (AC-T1) — module **`kesh-api/tests`** ;
- `docs/api-external.md` — **docs** ;
- `docs/manual/fr/{admin-manual,user-manual,marketing-brochure}.{tex,pdf}` — **manuels FR** ;
- `README.md`, `website/roadmap.html`, `CHANGELOG.md` — **textes publics** ;
- `_bmad-output/…` (fiche, registre, `sprint-status.yaml`, prompts ; l'epic, annoté à la validation P1, R8).

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

1. **Qui joue le parcours, et comment le prouver ?** Il demande la station et une démonstration. Trois façons :
   (a) **toi**, en session interactive, en suivant le PDF — preuve par tes captures, le transcript exporté et le journal
   d'audit ; (b) **l'agent de développement**, qui lance Claude Code en mode non interactif (`claude -p`) contre une
   démonstration, avec une configuration jetable et ses seuls outils de démonstration — preuve par le journal
   d'exécution, le transcript complet, la capture des échanges (`socat`), les captures d'écran et l'export du journal
   d'audit ; (c) **un test automatisé côté serveur** seulement — il prouve le serveur, pas le client.
   *Ma recommandation : (b) pour tout le parcours, plus (c) pour la CI, et un rejeu court par toi à la recette de la
   0.16.0, depuis le PDF, des seuls gestes que l'agent ne peut pas jouer* (une vingtaine de minutes : `claude mcp add
   --scope local`, `/mcp`, un prompt invoqué par sa commande `/…`, et ce que l'agent dit d'un envoi en attente quand une
   personne est là). (c) seul ne dit rien de Claude Code ; (a) seul coûte ton temps à chaque correction du manuel et
   n'est pas rejouable par un agent. Les coûts de (b) : le parcours consomme l'usage de **ton** compte Claude — l'agent
   lance `claude` avec ton authentification, faute de quoi il faudrait une clé d'API facturée à part — : cinq sessions
   (quatre au parcours, une à la restauration), autant au rejeu depuis le PDF, une de plus pour les pièges, chacune
   bornée à huit tours (et à un budget si `claude -p` en offre un) ; ces sessions s'inscrivent dans ton historique de
   Claude Code (sous le dossier `kesh-gate-logs/3211/cwd`), et ton `~/.claude/CLAUDE.md`, tes hooks et tes skills s'y
   appliquent ; et si le système de permissions refuse de lancer `claude` depuis l'agent, l'agent s'arrête et (a)
   devient la seule voie.
2. **L'oracle du test de contrat : des clauses recopiées à la main, ou le schéma officiel ?** La fiche fait écrire
   chaque attendu depuis la phrase citée de la spécification `2025-11-25`, avec des listes closes de champs (AC-B2,
   AC-B4). L'alternative : versionner le `schema.json` officiel de la révision et valider chaque réponse avec une
   dépendance de test (`jsonschema`, absente du dépôt aujourd'hui, avec ses propres dépendances). *Ma recommandation :
   les clauses à la main pour la 0.16.0* — pas de dépendance, et chaque attendu se lit dans le test ; le schéma officiel
   se reprend le jour où Kesh réexamine le SDK (`rmcp`, Q8 de la spike). Le prix : une lecture fausse de la
   spécification passerait (§ *Ce que les gardes ne voient pas*).
3. **Jouer un libellé piégé avec le vrai client ?** (Q2 de la fiche 32-9.) Les pièges vivent derrière une fonctionnalité
   de compilation des seuls tests ; les jouer demande un binaire compilé avec elle, pour **un** appel de plus.
   *Ma recommandation : oui, un seul appel, sur un binaire de parcours, sans toucher la démonstration publique* — c'est
   la seule occasion d'observer ce que fait un agent réel devant une instruction cachée dans un libellé. Le constat
   s'écrit, rien ne se corrige ici.
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
  ce qu'ils admettent de plus) ; le caractère facultatif d'`isError` et de la session (AC-B2) ; la clause sur une
  entrée que le serveur ne peut accepter (K21, « MUST return an HTTP error status code »). Texte cité, date de
  consultation ; chaque assertion de K1 à K22 rangée en `CLAUSE` ou `DECISION_KESH` selon ce texte.
- **Claude Code** (rien n'en a été exécuté par la fiche ni par ses lentilles, hors la version, fait 14) :
  `claude --version` ; `claude --help` (options `-p`, `--mcp-config`, `--strict-mcp-config`, `--allowedTools`,
  `--disallowedTools`, `--output-format`, `--verbose`, `--resume`, `--max-turns`, une option de budget) ; sa page MCP
  (spike § 2.1) relue : la forme de la commande d'un prompt MCP et son service en mode `-p` (P5 (c)) ; le message
  `init` du `stream-json` (serveurs et outils listés, P3, P4) ; que `--resume` retrouve une session depuis le même
  dossier de lancement.
- **L'amorçage** (P0) : la forme du corps de `POST /api/v1/journal-entries` (montants en chaîne ou en nombre, ligne
  au crédit seul) au code de la route ; l'écran des clés d'API ; la forme des montants dans le texte de `kesh_bilan`
  (code de la 32-9) — d'où le motif de l'AC-A5 (b) ; l'argument `fiscalYearId` de `kesh_bilan`.
- **Les prompts** : qu'une version écrite par l'Administrateur est en vigueur sans approbation (P5 (b), fiche 32-10b
  AC-7 et AC-9, au code).
- **Les tests** : `tests/common/mcp.rs` (aides de routeur, de clé, de société, `MockConnectInfo`) ;
  `tests/common/manuel.rs` (`normaliser`) ; `audit_route_registry.rs` (partition, l. 877-929 à `ed981c7d`) ; les listes
  fermées de `textes_coherents.rs` (G18, G19) et de `mcp_gardes_lexicales.rs` qu'un fichier neuf peut toucher ; le test
  de la matrice de la 32-9 (ancre `% matrice-mcp`).
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
- **Les réponses de Guy** aux points 1 à 5 ci-dessus, et à la Q2 de la fiche 32-9 (point 3).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-27 | création | Fiche rédigée en autonomie (Opus 5.5) contre `ed981c7d`, après lecture de l'epic (§ 32-11 et notes datées), de la spike (§ 1, 2, 6), des fiches 32-6, 32-6c, 32-8 (dans ce worktree) et des fiches 32-7, 32-9, 32-10a, 32-10b (dans leurs worktrees, en lecture seule), de la 31-4 et de la rétrospective de l'Epic 31, et des issues #759, #761, #765, #768, #770, #773, #776 (lecture). Constats au relevé : aucun test ne porte le contrat du protocole (fait 3) ; Kesh ne journalise pas la révision demandée (fait 6) ; le compte des routes du manuel est faux de deux (fait 8). Registre C-32-11-1 à C-32-11-24 ; `sprint-status.yaml` : `32-11-brancher-un-agent: ready-for-dev` ; prompt de validation P1 `3211-validate-prompt-p1.md` (Sonnet, lentilles R et F). Décomptes (recomptés sur la fiche par `grep`) : **33 AC** (A 8, B 6, C 9, D 5, E 3, T 2), **11 tâches** (T-0 à T-10), **16 mutations** M1 à M16 plus MT1, MT2, MF1, **22 cas** de contrat (K1 à K22), **11 gestes** de parcours (P1 à P11) plus P-pièges conditionnel, **15 faits**, **5 points** pour Guy, **4 modules**. |
| 2026-10-27 | validation P1 | Lentilles R et F (Sonnet, contexte frais) ; rapports `kesh-gate-logs/3211-validate-p1-{R,F}.md`. **Trend : 0 CRITICAL / 1 HIGH / 5 MEDIUM / 16 LOW** (R 0/1/1/7, F 0/0/4/9). Remédiation par l'orchestrateur de la boucle (agent de spécification, Opus 5.5), chaque prémisse relue au dépôt. **R1 (HIGH)** — vrai (`grep -nE 'journal_entries\|contacts\|invoices' crates/kesh-seed/src/lib.rs` → rien) : fait 16 ; geste **P0** (amorçage : clé d'API `read-write` créée à l'écran, trois `POST /api/v1/journal-entries` aux montants fixés, clé révoquée, bilan constaté à l'écran ; route `lib.rs:486`, ouverte aux clés `read-write`, `api-external.md:218`, `:265`, `:269`) ; AC-A5 (b) cherche 20 432.05 et 17 345.10 avec un **témoin positif** (le même `grep` sur `socat.log` après P7) ; `socat` lancé après P0 ; C-32-11-25. **F1** — vrai (l'accusé ne porte aucune donnée, `politique.rs` ; `kesh_bilan` prend `fiscalYearId`, fiche 32-9 l. 356) : la consigne de P6 est écrite et donne l'identifiant lu en P0 ; **un** envoi au parcours, **un** à la restauration ; sessions S1 à S4 comptées. **F2** — montage du client à P3 : dossier de lancement neuf hors dépôt (`kesh-gate-logs` n'est dans aucun dépôt, aucun `CLAUDE.md` parent, vérifié par `git rev-parse` et `ls`), `--strict-mcp-config`, aucun dossier de configuration jetable (l'authentification de Guy reste, ses sessions s'écrivent dans son magasin : déclaré, et « ne jamais écrire dans `~/.claude*` » retiré au profit de « jamais lui-même »), `--max-turns 8` et un budget si offert, arrêt sans plafond ; `claude mcp add` ne s'éprouve plus (la condition de l'AC-A8 retirée) ; C-32-11-5 amendé, C-32-11-26 ; Q1 dit le coût. **F3** — la liste de l'AC-A2 est la seule source : P3 est le montage, P5 en trois sous-gestes, P3-direct nommé, la variante « route de décision » des Dev Notes retirée (un geste d'écran que l'agent ne peut jouer l'arrête). **Réfuté en partie** : « créer une version personnalisée et l'approuver » — une version écrite par l'Administrateur est en vigueur sans approbation, *Approuver* ne portant que sur une version proposée par un agent (fiche 32-10b, AC-7 l. 328-343, AC-9 l. 384-395) : P5 (b) constate la version en vigueur, et se réécrit « l'approuver » si le code fusionné l'exige (étape 0). **F4, R2, R3** — l'oracle : `CLAUSE` (clés requises, types, valeurs imposées) et `DECISION_KESH` (listes closes K1, K7, K14 ; `isError` K8 et M10 ; session AC-B3 et M5 ; K10 ; K12 ; K17 ; K19 ; K22) par assertion ; attendus en littéraux ; aides `attendu_*` (`common/mcp.rs:626-663`, vérifié : elles appellent `mcp::textes::texte_envoi_*`) interdites au contrat, prouvé par les mutations **M17** et **M18** (sans garde de texte nouvelle, que l'en-tête de la fiche exclut) ; le schéma officiel se constate à l'étape 0 (prémisses des lentilles sur `isError` facultatif et les propriétés supplémentaires **non vérifiées** ici : aucun accès réseau) ; C-32-11-7 et C-32-11-8 amendés, C-32-11-27. **LOW intégrés** : M11 (K12 reste vert, même branche) ; M16 réécrite (`autoriser` rend `Envoi` pour `Required`, `politique.rs:106-111`, vérifié) ; M5 touche `reponse::rendre` ; `enveloppe_conforme(&Rendu, Option<i64>)` (F6) ; K21 à 400 avec sa clause à citer, sinon retiré et **angle mort** écrit (R4, F7 — la phrase « MUST return an HTTP error status code » vient de la mémoire des deux lentilles, non vérifiée) ; MF1 et l'en-tête de provenance (heure, longueur de la ligne `socat`), la fabrication dite angle mort (F8) ; `configuration_transmise.rs` (`MANUEL_ADMIN`, l. 2619) au gate ciblé, et les gardes `textes_coherents.rs:411`, `:579` à l'AC-T2 (F9) ; `KESH_TEST_MODE` retiré du montage (`lib.rs:1309-1314`, `main.rs:279`) et le build du frontend au montage (F10) ; ce qui part avec un retrait de l'AC-T1 (F11 ; l'AC-T1 reste) ; K10 par la route de décision (F12, `lib.rs:380`, `common/mcp.rs:666`, `:685`) ; `website/roadmap.html:43` au point 5 avec `README.md:232`, et l'AC-D3 qui lit 43, 223, 372, 375 (R5, F13 ; `grep -n -i mcp website/roadmap.html` → 43, 223, 372, 375) ; AC-C9 sous `LC_ALL=fr_FR.UTF-8`, `L.{1,3}agent`, règle de la césure de la 32-7 (R6) ; AC-C6 (R7) : **réfuté en partie** — #765 (5) est **déjà dit en entier** à `admin-manual.tex:2238` (« une sauvegarde prise entre-temps le porte encore »), non à moitié, et #773 (9) à `:2949` ; restent #768 (1) et #773 (8) ; l'epic annoté, l. 522 (R8 ; texte d'origine laissé) ; fait 2 (`registre.rs:46`) et fait 3 (littéraux de `initialize_ping_prompts`) (R9). **Écarts des lentilles** : la lentille R a lancé `claude --version` (sortie `2.1.311 (Claude Code)`), un `awk` sans programme redirigé vers `/dev/null` et une substitution de processus, toutes interdites par le prompt et sans effet sur le dépôt ; la version est **à reconfirmer à l'étape 0** (fait 14). La lentille F n'en déclare aucun. **Décomptes** (recomptés sur la fiche et le registre par `grep -c`) : **33 AC** (A 8, B 6, C 9, D 5, E 3, T 2), **11 tâches**, **18 mutations** M1 à M18 plus MT1, MT2, MF1, **22 cas** K1 à K22, **12 gestes** P0 à P11 plus deux conditionnels (P3-direct, P-pièges), **16 faits**, **5 points** pour Guy, **27 choix** (C-32-11-25 à 27 neufs ; C-32-11-2, 5, 7, 8 et 23 amendés par une ligne datée), **4 modules**. Symptômes grepés après patch : `ce que l'agent choisit`, `Ne jamais`, `.claude*`, `M1 à M16`, `onze`, `route de décision`, `déplace le dossier` → aucun résidu hors du Change Log (l'entrée de création, laissée telle quelle, garde « M1 à M16 ») ; « route de décision » reste à K10, où elle est le moyen prescrit (F12), non une variante de geste. Prompt de la P2 : `3211-validate-prompt-p2.md` (Opus, lentilles R et F). |
