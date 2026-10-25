# Prompt — validation P3, Story 32-6 (l'endpoint `/mcp` : protocole, garde, lecture seule, filtre, journal)

*Versionné le 2026-10-25. Passe **complète** (troisième passe ; P1 Sonnet 0/0/11/22, P2 Opus 0/0/9/23, remédiés — **sept
des neuf MEDIUM de la P2 étaient nés de la remédiation P1**). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet** (modèle différent de la P2, Opus). Lecture seule stricte, **sans `gh`**.*

Dépôt `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6` (partie de `main` `b723e3d9`, la 32-5a fusionnée ; les
32-5b, 32-5d et 32-5c ne le sont pas encore — la fiche en dépend et le dit). Fiche :
`_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (deux parties : 32-6a, 32-6b) ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326.md` (C-32-6-1 à 37 ; les choix révisés portent une
annotation datée). **Rapports de la P1 et de la P2**, à lire pour savoir ce qui a été remédié — **non pour les reprendre
crus** : `/home/gcorbaz/devel/kesh-gate-logs/326-validate-p{1,2}-{R,F}.md` ; les entrées « validation P1 » et
« validation P2 » du Change Log de la fiche disent ce qui a été fait de chacun, et un écart (F7 de la P1, colonne de
rejeu). Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-6* et ses *Reports de la
32-5*, § *Ce que le code dit déjà*, § *Points d'attention*, § *Réponses de Guy*) ; spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (**en entier**, dont § 5.3 E1 à E6, § 5.3.3, § 6, § 6.2, § 7 et
ses **réponses** Q10, Q11, Q12) ; fiche `32-5-cles-et-reglages-mcp.md` (AC-b3, AC-b4, AC-c4, § *Ce que reçoivent les
stories suivantes*, § *Pour la 32-5b*) et `epic-32-choix-autonomes-325.md` (C-32-5-5, 7 à 11, 14, 19, 37) ; fiche
`32-3-services-motif.md` (gardes v-1, v-2, vi ; C-32-3-5, C-32-3-30). Règles : `CLAUDE.md` (§ *Review Iteration Rule*
et *La passe ciblée* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW ; une remédiation de
l'orchestrateur ou d'un agent n'a aucun privilège de justesse —, *Inventorier les sites NON RÉSOLUS*, *Règle de
splitting préventif*, *Pendant une boucle de revue* et son exception `kesh-db`, *Migration breaking policy*, *Le prompt
d'une passe doit NOMMER le manuel*, *Recompter ses propres comptes rendus*, *Propagation post-patch*),
`docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : **Q11** — lecture seule **applicative** plus mode de
session `READ ONLY` ; ni second utilisateur MariaDB, ni variable `.env` (un champ d'`AppState` n'est **pas** interdit ;
la fiche a choisi un sous-routeur à état propre, C-32-6-31) ; « par construction » ne vaut pas pour la lecture seule ;
**Q10 (c)** et **Q12** — la poignée et sa table sont la 32-8 ; **réseau** — local, filtre de défense en profondeur
désactivable, `X-Forwarded-For` des seuls mandataires déclarés, Traefik devant une partie des conteneurs ; désactivé
par défaut et alors **indiscernable d'une route absente** ; journal sans arguments ni données ; au plus cinq modules par
partie ; aucune donnée réelle. Le découpage 32-6a / 32-6b est **confirmé** par l'orchestrateur. Ce que la fiche
**prescrit** n'existe pas encore : son absence n'est pas un défaut.

⚠️ **Le motif mesuré de ce dépôt, et il s'est vérifié en P2** : la sévérité se déplace vers ce qu'on vient d'écrire.
La remédiation P2 a choisi, sur consigne de l'orchestrateur, **la conception la plus simple qui ferme la classe** plutôt
qu'une garde de plus. **Cherche d'abord les défauts nés de cette remédiation** : la liste blanche des erreurs (AC-a18),
la sortie unique `reponse::rendre` (AC-a31, C-32-6-37), `politique::autoriser` et son jeton (AC-a15, C-32-6-36),
`EtatMcp` sans `AppState` et la garde `mcp_sans_pool` (AC-a1, AC-a30), l'extraction de **tout** `fallback(` (AC-a6),
`after_release` à la place de `before_acquire` (AC-a25, C-32-6-35) ; et les renvois, mutations et annotations du
registre qui ne les auraient pas suivis. Ne propose pas une garde de plus là où une conception plus simple fermerait la
classe : dis-le.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Les faits 1 à 12 et le § *Lignes de code lues et citées*** : chaque numéro de ligne (les citations de
   `services/acces.rs`, `audit_route_registry.rs`, `errors.rs`, `sqlx-core-0.8.6/src/pool/inner.rs`,
   `axum-0.8.9/src/extract/connect_info.rs` sont neuves).
2. **Le canal et la sortie** (C-32-6-31, C-32-6-37, AC-a1, AC-a31) : `EtatMcp` (`config`, `lecture`, `trace`,
   `registre`, `statique`) suffit-il à tout ce que le transport fait (Origin, authentification, règle « servi », délai,
   remise au service statique) ? `ReponseMcp` peut-elle porter la réponse du service statique sans contenir de
   `Response` (comment `rendre` la produit-il) ? La garde d'inventaire des sites (`AppError`, `IntoResponse`,
   `into_response`) est-elle une liste close et rejouable ? `Result<ConnectInfo<SocketAddr>, ExtensionRejection>` et
   `Request` en dernier extracteur restent-ils justes ?
3. **Le registre des routes (AC-a6, C-32-6-32, AC-b6)** : relis `extract_counted`, `appels_non_handlers` et leurs tests ;
   l'extraction de **tout** `fallback(`, avec ses arguments non `routes::` à l'inventaire des non-résolus moins la liste
   écrite des sites `ServeFile::new(`, est-elle juste et réalisable (rustfmt qui coupe la ligne, rognage de l'argument,
   `fallback_service(`, l'argument `ServeFile::new(format!(…` coupé à la première `)`) ? Le témoin discriminant
   d'`appels_non_handlers` l'est-il vraiment ? Les compteurs nommés (`:768`, `:770-771`, `:788`, `:789-791`,
   `:793-797`, `:1119-1127`, `:1129`, doc `:27-32`) sont-ils **tous** ceux qui bougent, et les deltas justes en 32-6a
   puis en 32-6b ? `every_exemption_names_the_issue_that_follows_it` (`:801-814`) accepte-t-il le motif proposé ?
   L'écart à F7 (rejeu `SansEcritureAuJournal`) tient-il au code (`:284-300`, `:326`) ?
4. **Le réseau et l'`Origin`** (AC-a7 à AC-a10) : l'algorithme de `X-Forwarded-For` contre (a)–(r) ; la
   normalisation appliquée des deux côtés contre la forme stockée par la 32-5c (C-32-5-19) ; `public_base_url`
   (`config.rs:334`, `:1089-1091`) et le cas invalide ; l'avertissement dédupliqué de l'AC-a8 — réalisable sans état
   partagé neuf, ou l'état est-il nommé ?
5. **La lecture seule et les erreurs** (AC-a18, AC-a20, AC-a25 à AC-a28) : `after_release` dans `sqlx-core-0.8.6`
   (`pool/options.rs:492`, `pool/connection.rs:270-320`) — est-il appelé à **chaque** restitution, transaction comprise
   (`Transaction` rendue par `commit`/`rollback`/`drop`), et une connexion fermée ne peut-elle jamais être reprise ? Les
   tests (e) et (e bis) sont-ils réalisables tels qu'écrits avec `max_connections(1)` et une connexion fermée de façon
   asynchrone ? La garde de l'AC-a27 (nom `tx_read_only` interdit en entier, exceptions écrites) ; la liste blanche de
   `depuis_app_error` contre l'énumération réelle (`errors.rs:139-1182`, `kesh-db/src/errors.rs`) et la forme réelle
   de l'« introuvable » des services.
6. **Le protocole et la frontière 32-8** contre la spike § 1.3, § 5.3.1, § 5.3.3 et § 6 : chaque exigence destinée à la
   32-6 a-t-elle encore son AC ? `autoriser` / `appliquer` / `Autorisation` : la frontière écrite (la 32-8 change les
   deux fonctions, signatures comprises, et leurs deux sites d'appel) est-elle juste, et les mutations Ma23 et Ma23b
   éprouvent-elles ce qu'elles disent ?
7. **Le manuel et la documentation** : la fiche dit que la 32-6 n'écrit rien au manuel. Vérifie :
   `grep -n "MCP" docs/manual/fr/*.tex docs/api-external.md README.md website/*.html` ; et **le PDF aplati** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}(MCP|journal d.audit|clé API).{80}"`,
   idem `user-manual.pdf`. La liste laissée à la 32-7 et à la 32-11 (§ *Le manuel*) est-elle complète au regard des
   AC remédiés — dont `README.md:233` et `website/roadmap.html:375`, faux sous une clé `dispensée` ?

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Prémisses** neuves de la remédiation : chaque « établi au code » ajouté en P2 tient-il (`after_release`,
   `errors.rs`, `capture_rejeu.rs`, `routing/mod.rs`) ? Une remédiation repose-t-elle sur un finding P2 sans
   vérification ?
2. **Découpage** : recompte les modules — 32-6a 4, 32-6b 5 (`routes` par `audit_route_registry.rs`), six distincts,
   neuf en somme ? Les fichiers touchés de chaque partie restent-ils dans ses modules (le `repositories/mcp_keys.rs`
   conditionnel compris) ? L'état intermédiaire après la 32-6a ouvre-t-il un accès ?
3. **L'inventaire clos** (N1 à N27) : cherche un chemin **qui n'y est pas** — en particulier ceux qu'ouvrent les
   nouveautés de la P2 : `EtatMcp.config` (le `Config` porte-t-il un secret ou une URL de base qu'un outil pourrait
   rendre ?), `Autorisation`, `ReponseMcp`, `Donnees::nouvelles` restreint à `src/mcp/outils/`, la liste écrite des
   sites `ServeFile::new(`. Chaque ligne est-elle **fermée par ce qu'elle dit** ?
4. **Sécurité de la garde** : l'ordre des contrôles (Dev Notes) — le 401 fixe pour une clé d'une société qui ne sert pas
   (C-32-6-3 révisé) crée-t-il un autre oracle (temps, `last_used_at`) ? Les réponses dites égales (route absente, 401,
   outil inconnu) le sont-elles **octet pour octet** dans la conception ? Le corps de 3 Mio : 401 avant lecture, 413
   après, route absente quand désactivé — la conception le tient-elle ?
5. **Tests et mutations** : chaque AC est-il vérifiable tel qu'écrit ; chaque mutation Ma1 à Ma49 (et Ma23b), Mb1 à Mb6
   **réalisable** et tuée par le test **nommé** (une passerait-elle à vide ? une ne compile-t-elle pas ?) ; les
   mutations exigées (PAT accepté, filtrage de `tools/list`, filtre d'adresse, `Origin`, lecture seule) sont-elles
   toujours là ?
6. **Décomptes** : recompte à la source — 37 AC (31 / 6), 11 tâches (7 / 4), 55 mutations (49 / 6), C-32-6-1 à 37, et
   les chiffres de l'entrée « validation P2 » du Change Log (dont « sept des neuf MEDIUM nés de la remédiation P1 » ;
   cohérence du total avec sa ventilation, `CLAUDE.md` § *Recompter*).
7. **Ce que la fiche omet** : un report de la 32-5 ou de la spike, un compteur, une garde ou un site que la remédiation
   a rendu nécessaire et qu'elle ne nomme pas ; les « points à soumettre à Guy » (le 2 bis est-il posé honnêtement, ses
   conséquences dites ?) ; un choix pris en autonomie qui reviendrait à Guy.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée ; pour un finding né de la remédiation P2 (ou P1), dis-le. Un bilan d'en-tête **cohérent
avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R7, F1 à F7) — sans elle, « 0 finding » ne compte
pas comme passe. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/326-validate-p3-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/326-validate-p3-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`** et
**`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`,
**`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
et **`gh` sous toutes ses formes**. `pdftotext` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de
`~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre. Autorisés : lecture, `grep`,
`sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`,
`head`, `tail`, `awk`, `wc`, `uniq` vers la sortie standard.
